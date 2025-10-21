use crate::Res;
use crate::sense_voice_small::encoder::{MultiHeadedAttentionSANM, PositionwiseFeedForward};
use candle_core::Tensor;
use candle_nn::{Dropout, LayerNorm, Linear, Module, VarBuilder};

#[derive(Debug)]
pub struct EncoderLayerSANM {
    /// Self attention module
    self_attn: MultiHeadedAttentionSANM,
    /// Feed forward module
    feed_forward: PositionwiseFeedForward,
    /// Layer normalization for attention
    norm1: LayerNorm,
    /// Layer normalization for feed forward
    norm2: LayerNorm,
    /// Dropout module
    dropout: Dropout,
    /// Input size
    in_size: usize,
    /// Hidden size
    size: usize,
    /// Whether to normalize before the computation
    normalize_before: bool,
    /// Whether to concatenate after attention
    concat_after: bool,
    /// Concatenation linear layer (when concat_after is true)
    concat_linear: Option<Linear>,
}

impl EncoderLayerSANM {
    /// Construct an EncoderLayerSANM object
    ///
    /// # Arguments
    /// * `in_size` - Input size
    /// * `size` - Hidden size
    /// * `self_attn` - Self attention module
    /// * `feed_forward` - Feed forward module
    /// * `dropout_rate` - Dropout rate
    /// * `normalize_before` - Whether to normalize before the computation
    /// * `concat_after` - Whether to concatenate after attention
    /// * `stochastic_depth_rate` - Stochastic depth rate
    pub fn new(
        in_size: usize,
        size: usize,
        self_attn: MultiHeadedAttentionSANM,
        feed_forward: PositionwiseFeedForward,
        dropout_rate: f32,
        normalize_before: bool,
        concat_after: bool,
        vb: VarBuilder,
    ) -> Res<Self> {
        let norm1 = candle_nn::layer_norm(in_size, 1e-5, vb.pp("norm1"))?;
        let norm2 = candle_nn::layer_norm(size, 1e-5, vb.pp("norm2"))?;
        let dropout = Dropout::new(dropout_rate);

        let concat_linear = if concat_after {
            Some(candle_nn::linear(
                size + size,
                size,
                vb.pp("concat_linear"),
            )?)
        } else {
            None
        };

        Ok(Self {
            self_attn,
            feed_forward,
            norm1,
            norm2,
            dropout,
            in_size,
            size,
            normalize_before,
            concat_after,
            concat_linear,
        })
    }

    /// Compute encoded features
    ///
    /// # Arguments
    /// * `x` - Input tensor (batch, time, size)
    /// * `mask` - Mask tensor for the audio (batch, time)
    /// * `cache` - Cache tensor of the audio (batch, time - 1, size)
    ///
    /// # Returns
    /// * Output tensor (batch, time, size)
    /// * Mask tensor (batch, time)
    pub fn forward(&self, x: &Tensor) -> Res<Tensor> {
        let stoch_layer_coeff = 1.0;

        let residual = x.clone();
        let mut x = x.clone();
        if self.normalize_before {
            x = self.norm1.forward(&x)?;
        }

        if self.concat_after {
            let attn = self.self_attn.forward(&x)?;
            let x_concat = Tensor::cat(&[&x, &attn], 2)?;

            if let Some(concat_linear) = &self.concat_linear {
                x = if self.in_size == self.size {
                    let concat_res = concat_linear.forward(&x_concat)?;
                    (&residual + (stoch_layer_coeff * &concat_res)?)?
                } else {
                    concat_linear.forward(&x_concat)?
                };
            }
        } else {
            let attn = self.self_attn.forward(&x)?;
            let dropout_attn = self.dropout.forward(&attn, false)?;

            if self.in_size == self.size {
                x = (&residual + (stoch_layer_coeff * &dropout_attn)?)?;
            } else {
                x = (stoch_layer_coeff * &dropout_attn)?;
            }
        }

        if !self.normalize_before {
            x = self.norm1.forward(&x)?;
        }

        let residual = x.clone();
        if self.normalize_before {
            x = self.norm2.forward(&x)?;
        }

        let ff_res = self.feed_forward.forward(&x)?;
        let dropout_ff = self.dropout.forward(&ff_res, false)?;
        x = (&residual + (stoch_layer_coeff * &dropout_ff)?)?;

        if !self.normalize_before {
            x = self.norm2.forward(&x)?;
        }

        Ok(x)
    }
}
