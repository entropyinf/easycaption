use crate::Res;
use candle_core::Tensor;
use candle_nn::{
    Conv1d, Conv1dConfig, Dropout, Linear, Module, VarBuilder, conv1d_no_bias, linear,
};
use std::time::Instant;

#[derive(Debug)]
pub struct MultiHeadedAttentionSANM {
    /// The dimension of each head
    d_k: usize,
    /// The number of heads
    h: usize,
    /// Linear layer for output transformation
    linear_out: Linear,
    /// Combined linear layer for Q, K, V transformations
    linear_q_k_v: Linear,
    /// FSMN convolution block
    fsmn_block: Conv1d,
    /// Padding values for FSMN
    left_padding: usize,
    /// Padding values for FSMN
    right_padding: usize,
    /// Dropout layer
    dropout: Dropout,
}

impl MultiHeadedAttentionSANM {
    /// Creates a new MultiHeadedAttentionSANM instance
    ///
    /// # Arguments
    /// * `n_head` - The number of heads
    /// * `in_feat` - The audio feature size
    /// * `n_feat` - The feature size
    /// * `dropout_rate` - Dropout rate
    /// * `kernel_size` - Kernel size for FSMN
    /// * `sanm_shfit` - Shift for SANM (default: 0)
    /// * `vb` - VarBuilder for creating layers
    pub fn new(
        n_head: usize,
        in_feat: usize,
        n_feat: usize,
        dropout_rate: f32,
        kernel_size: usize,
        sanm_shfit: usize,
        vb: VarBuilder,
    ) -> Res<Self> {
        assert_eq!(n_feat % n_head, 0, "n_feat must be divisible by n_head");

        // We assume d_v always equals d_k
        let d_k = n_feat / n_head;
        let h = n_head;

        let linear_out = linear(n_feat, n_feat, vb.pp("linear_out"))?;
        let linear_q_k_v = linear(in_feat, n_feat * 3, vb.pp("linear_q_k_v"))?;

        // FSMN block - Conv1d with groups=n_feat and no bias
        let fsmn_block = conv1d_no_bias(
            n_feat,
            n_feat,
            kernel_size,
            Conv1dConfig {
                groups: n_feat,
                stride: 1,
                padding: 0,
                ..Conv1dConfig::default()
            },
            vb.pp("fsmn_block"),
        )?;

        // Calculate padding
        let left_padding = (kernel_size - 1) / 2 + sanm_shfit;
        let right_padding = kernel_size - 1 - left_padding;

        let dropout = Dropout::new(dropout_rate);

        Ok(Self {
            d_k,
            h,
            linear_out,
            linear_q_k_v,
            fsmn_block,
            left_padding,
            right_padding,
            dropout,
        })
    }

    /// Forward pass for FSMN
    fn forward_fsmn(
        &self,
        inputs: &Tensor,
    ) -> Res<Tensor> {
        let inputs = inputs.clone();

        // Transpose to (b, d, t)
        let x = inputs.transpose(1, 2)?;

        // Apply padding
        let x = x.pad_with_zeros(2, self.left_padding, self.right_padding)?;

        let x = self.fsmn_block.forward(&x)?;

        // Transpose back to (b, t, d)
        let x = x.transpose(1, 2)?;

        // Add residual connection
        let x = (x + inputs)?;

        let x = self.dropout.forward(&x, false)?;

        Ok(x)
    }

    /// Transform query, key and value
    fn forward_qkv(&self, x: &Tensor) -> Res<(Tensor, Tensor, Tensor, Tensor)> {
        let (b, t, _) = x.dims3()?;

        let q_k_v = self.linear_q_k_v.forward(x)?;
        let chunks = q_k_v.chunk(3, 2)?; // Split into 3 chunks along dim 2
        let q = &chunks[0];
        let k = &chunks[1];
        let v = &chunks[2];

        let q_h = q.reshape((b, t, self.h, self.d_k))?.transpose(1, 2)?; // (batch, head, time1, d_k)
        let k_h = k.reshape((b, t, self.h, self.d_k))?.transpose(1, 2)?; // (batch, head, time2, d_k)
        let v_h = v.reshape((b, t, self.h, self.d_k))?.transpose(1, 2)?; // (batch, head, time2, d_k)

        Ok((q_h, k_h, v_h, v.clone()))
    }

    /// Compute attention context vector
    fn forward_attention(
        &self,
        value: &Tensor,
        scores: &Tensor,
    ) -> Res<Tensor> {
        let attn = candle_nn::ops::softmax(&scores, 3)?;

        let p_attn = attn;

        let p_attn = self.dropout.forward(&p_attn, false)?;
        let x = p_attn.matmul(&value.contiguous()?)?; // (batch, head, time1, d_k)
        let x = x.transpose(1, 2)?.flatten_from(2)?; // (batch, time1, d_model)

        let out = self.linear_out.forward(&x)?; // (batch, time1, d_model)

        Ok(out)
    }

    /// Forward pass
    pub fn forward(
        &self,
        x: &Tensor,
    ) -> Res<Tensor> {
        let start = Instant::now();
        let (q_h, k_h, v_h, v) = self.forward_qkv(x)?;
        println!("forward qkv cost: {:?}", start.elapsed());

        let start = Instant::now();
        let fsmn_memory = self.forward_fsmn(&v)?;
        println!("forward fsmn cost: {:?}", start.elapsed());

        // Scale query
        let scale = (self.d_k as f32).powf(-0.5);
        let scale_tensor = Tensor::new(scale, q_h.device())?;

        let start = Instant::now();
        let q_h = q_h.broadcast_mul(&scale_tensor)?;
        println!("broadcast_mul cost: {:?}", start.elapsed());

        let mut k_h = k_h.transpose(2, 3)?;

        if k_h.device().is_metal() {
            k_h = k_h.contiguous()?;
        }

        let start = Instant::now();
        let scores = q_h.matmul(&k_h)?;
        println!("scores cost: {:?}", start.elapsed());

        let start = Instant::now();
        let att_outs = self.forward_attention(&v_h, &scores)?;
        println!("forward_attention cost: {:?}", start.elapsed());

        Ok((att_outs + fsmn_memory)?)
    }
}
