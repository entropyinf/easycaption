use crate::Res;

use crate::sense_voice_small::encoder::{
    EncoderLayerSANM, MultiHeadedAttentionSANM, PositionwiseFeedForward, SinusoidalPositionEncoder,
};
use candle_core::Tensor;
use candle_nn::{LayerNorm, Module, VarBuilder};

/// Configuration for SenseVoiceEncoderSmall
#[derive(Debug, Clone)]
pub struct EncoderConfig {
    /// Input size
    pub input_size: usize,
    /// Output size
    pub output_size: usize,
    /// Number of attention heads
    pub attention_heads: usize,
    /// Number of linear units
    pub linear_units: usize,
    /// Number of encoder blocks
    pub num_blocks: usize,
    /// Number of TP blocks
    pub tp_blocks: usize,
    /// Dropout rate
    pub dropout_rate: f32,
    /// Attention dropout rate
    pub attention_dropout_rate: f32,
    /// Kernel size
    pub kernel_size: usize,
    /// SANM shift
    pub sanm_shfit: usize,
    /// Whether to normalize before computation
    pub normalize_before: bool,
    /// Whether to concatenate after attention
    pub concat_after: bool,
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            input_size: 0,
            output_size: 0,
            attention_heads: 4,
            linear_units: 2048,
            num_blocks: 6,
            tp_blocks: 0,
            dropout_rate: 0.1,
            attention_dropout_rate: 0.0,
            kernel_size: 11,
            sanm_shfit: 0,
            normalize_before: true,
            concat_after: false,
        }
    }
}

#[derive(Debug)]
pub struct Encoder {
    /// Embedding module
    embed: SinusoidalPositionEncoder,
    /// First encoder layers
    encoders0: Vec<EncoderLayerSANM>,
    /// Main encoder layers
    encoders: Vec<EncoderLayerSANM>,
    /// TP encoder layers
    tp_encoders: Vec<EncoderLayerSANM>,
    /// Layer normalization after main encoders
    after_norm: LayerNorm,
    /// Layer normalization after TP encoders
    tp_norm: LayerNorm,
    /// Output size
    output_size: usize,
}

impl Encoder {
    /// Create a new SenseVoiceEncoderSmall instance
    ///
    /// # Arguments
    /// * `config` - Configuration for the encoder
    /// * `vb` - VarBuilder for creating layers
    pub fn new_with_config(cfg: EncoderConfig, vb: VarBuilder) -> Res<Self> {
        // Create embedding module
        let embed = SinusoidalPositionEncoder;

        let create_layer =
            |input_size: usize, output_size: usize, vb: VarBuilder| -> Res<EncoderLayerSANM> {
                let self_attn = MultiHeadedAttentionSANM::new(
                    cfg.attention_heads,
                    input_size,
                    output_size,
                    cfg.attention_dropout_rate,
                    cfg.kernel_size,
                    cfg.sanm_shfit,
                    vb.pp("self_attn"),
                )?;

                let positionwise_layer = PositionwiseFeedForward::new(
                    cfg.output_size,
                    cfg.linear_units,
                    cfg.dropout_rate,
                    vb.pp("feed_forward"),
                )?;

                let encoder_layer = EncoderLayerSANM::new(
                    input_size,
                    output_size,
                    self_attn,
                    positionwise_layer,
                    cfg.dropout_rate,
                    cfg.normalize_before,
                    cfg.concat_after,
                    vb,
                )?;

                Ok(encoder_layer)
            };

        let vb = vb.pp("encoder");

        // Create first encoder layers (1 block)
        let encoders0 = {
            let mut encoders0 = Vec::new();
            let vb = vb.pp("encoders0");
            for i in 0..1 {
                encoders0.push(create_layer(cfg.input_size, cfg.output_size, vb.pp(i))?);
            }

            encoders0
        };

        // Create main encoder layers (num_blocks - 1 blocks)
        let encoders = {
            let mut encoders = Vec::new();
            let vb = vb.pp("encoders");
            for i in 0..(cfg.num_blocks - 1) {
                encoders.push(create_layer(cfg.output_size, cfg.output_size, vb.pp(i))?);
            }

            encoders
        };

        // Create TP encoder layers
        let tp_encoders = {
            let mut tp_encoders = Vec::new();
            let vb = vb.pp("tp_encoders");
            for i in 0..cfg.tp_blocks {
                tp_encoders.push(create_layer(cfg.output_size, cfg.output_size, vb.pp(i))?);
            }

            tp_encoders
        };

        // Create normalization layers
        let after_norm = candle_nn::layer_norm(cfg.output_size, 1e-5, vb.pp("after_norm"))?;
        let tp_norm = candle_nn::layer_norm(cfg.output_size, 1e-5, vb.pp("tp_norm"))?;

        Ok(Self {
            embed,
            encoders0,
            encoders,
            tp_encoders,
            after_norm,
            tp_norm,
            output_size: cfg.output_size,
        })
    }

    /// Forward pass
    ///
    /// # Arguments
    /// * `xs_pad` - Padded audio sequences (batch, time, size)
    /// * `ilens` - Input sequence lengths (batch,)
    ///
    /// # Returns
    /// * Output tensor (batch, time, size)
    /// * Output sequence lengths (batch,)
    pub fn forward(&self, xs_pad: &Tensor) -> Res<Tensor> {
        // Scale audio
        let xs_pad = (xs_pad * (self.output_size as f64).sqrt())?;

        // Apply embedding
        let mut xs_pad = self.embed.forward(&xs_pad)?;

        // Forward through first encoder layers
        for encoder_layer in &self.encoders0 {
            xs_pad = encoder_layer.forward(&xs_pad)?;
        }

        // Forward through main encoder layers
        for encoder_layer in &self.encoders {
            xs_pad = encoder_layer.forward(&xs_pad)?;
        }

        // Apply normalization after main encoders
        xs_pad = self.after_norm.forward(&xs_pad)?;

        // Forward through TP encoder layers
        for encoder_layer in &self.tp_encoders {
            xs_pad = encoder_layer.forward(&xs_pad)?;
        }

        // Apply normalization after TP encoders
        xs_pad = self.tp_norm.forward(&xs_pad)?;

        Ok(xs_pad)
    }
}
