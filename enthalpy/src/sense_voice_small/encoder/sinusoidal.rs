use crate::Res;
use candle_core::{DType, Device, Tensor};

#[derive(Debug)]
pub struct SinusoidalPositionEncoder;

impl SinusoidalPositionEncoder {
    pub fn encode(
        &self,
        positions: &Tensor,
        depth: usize,
        dtype: DType,
        device: &Device,
    ) -> Res<Tensor> {
        let batch_size = positions.shape().dim(1)?;

        // Calculate logarithmic time scale increment
        let log_timescale_increment = Tensor::new(10000.0f32, device)?
            .log()?
            .div(&Tensor::try_from((depth / 2 - 1) as f32)?.to_device(device)?)?;

        // Calculate inverse time scales
        let inv_timescales = Tensor::arange(0., (depth / 2) as f32, device)?
            .to_dtype(dtype)?
            .mul(
                &log_timescale_increment
                    .neg()?
                    .to_dtype(dtype)?
                    .broadcast_as(depth / 2)?,
            )?
            .exp()?;

        // Reshape inverse time scales to match broadcast dimensions
        let inv_timescales = inv_timescales.reshape((1, 1, depth / 2))?;

        // Calculate scaled time
        let scaled_time = positions
            .to_dtype(dtype)?
            .reshape((1, batch_size, 1))?
            .broadcast_mul(&inv_timescales)?;

        // Calculate sine and cosine encodings and concatenate
        let sin_encoding = scaled_time.sin()?;
        let cos_encoding = scaled_time.cos()?;

        let encoding = Tensor::cat(&[sin_encoding, cos_encoding], 2)?;

        Ok(encoding)
    }

    /// Forward propagation, adding positional encoding to input tensor
    pub fn forward(&self, x: &Tensor) -> Res<Tensor> {
        let shape = x.shape();
        let (_, timesteps, input_dim) = (shape.dim(0)?, shape.dim(1)?, shape.dim(2)?);
        let device = x.device();
        let dtype = x.dtype();

        // Create position tensor [1, timesteps]
        let positions =
            Tensor::arange(1i64, (timesteps + 1) as i64, device)?.reshape((1, timesteps))?;

        // Generate positional encoding
        let position_encoding = self.encode(&positions, input_dim, dtype, device)?;

        Ok((x + position_encoding)?)
    }
}
