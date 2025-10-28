use crate::Res;
use crate::var_builder::{Linear, VarBuilder};
use candle_core::Tensor;
use candle_nn::Dropout;

pub struct PositionwiseFeedForward {
    w_1: Linear,
    w_2: Linear,
    dropout: Dropout,
}

impl PositionwiseFeedForward {
    /// Create a new PositionwiseFeedForward instance
    ///
    /// # Arguments
    /// * `idim` - Input dimension
    /// * `hidden_units` - Number of hidden units
    /// * `dropout_rate` - Dropout rate
    /// * `vb` - VarBuilder for creating linear layers
    pub fn new(in_dim: usize, hidden_units: usize, dropout_rate: f32, vb: VarBuilder) -> Res<Self> {
        let w_1 = vb.pp("w_1").linear(in_dim, hidden_units)?;
        let w_2 = vb.pp("w_2").linear(hidden_units, in_dim)?;
        let dropout = Dropout::new(dropout_rate);

        Ok(Self { w_1, w_2, dropout })
    }

    pub fn forward(&self, x: &Tensor) -> Res<Tensor> {
        let x = self.w_1.forward(x)?.relu()?;
        let x = self.dropout.forward(&x, false)?;
        let out = self.w_2.forward(&x)?;

        Ok(out)
    }
}
