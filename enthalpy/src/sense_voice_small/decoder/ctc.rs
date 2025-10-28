use crate::var_builder::{Linear, VarBuilder};
use crate::Res;
use candle_core::{Result, Tensor};

pub struct CTCLoss {
    ctc_lo: Option<Linear>,
}

impl CTCLoss {
    /// Create a new CTC instance
    ///
    /// # Arguments
    /// * `odim` - Output dimension
    /// * `encoder_output_size` - Encoder output size
    /// * `dropout_rate` - Dropout rate (0.0 ~ 1.0)
    /// * `ctc_type` - CTC type ("builtin", etc.)
    /// * `reduce` - Whether to reduce CTC loss to scalar
    /// * `ignore_nan_grad` - Whether to ignore NaN gradients
    /// * `extra_linear` - Whether to use an extra linear layer
    pub fn new(
        odim: usize,
        encoder_output_size: usize,
        extra_linear: bool,
        vb: VarBuilder,
    ) -> Res<Self> {
        let ctc_lo = if extra_linear {
            Some(vb.pp("ctc_lo").linear(encoder_output_size, odim)?)
        } else {
            None
        };

        Ok(Self { ctc_lo })
    }

    /// Apply log_softmax to frame activations
    ///
    /// # Arguments
    /// * `hs_pad` - 3D tensor (B, Tmax, eprojs)
    /// # Returns
    /// * 3D tensor with log softmax applied (B, Tmax, odim)
    pub fn log_softmax(&self, hs_pad: &Tensor) -> Result<Tensor> {
        if let Some(ctc_lo) = &self.ctc_lo {
            candle_nn::ops::log_softmax(&ctc_lo.forward(hs_pad)?, 2)
        } else {
            candle_nn::ops::log_softmax(hs_pad, 2)
        }
    }
}
