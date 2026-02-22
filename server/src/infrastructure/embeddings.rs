use anyhow::{Context, Result};
use image::DynamicImage;
use ndarray::Array4;
use ort::{Environment, Session, SessionBuilder, Value};
use std::sync::Arc;

pub struct EmbeddingModel {
    session: Session,
}

impl EmbeddingModel {
    pub fn new(model_path: &str) -> Result<Self> {
        let env = Arc::new(Environment::builder().with_name("arcface").build()?);
        let session = SessionBuilder::new(&env)?.with_model_from_file(model_path)?;
        Ok(Self { session })
    }

    pub fn generate(&mut self, face_img: &DynamicImage) -> Result<Vec<f32>> {
        let resized = face_img.resize_exact(112, 112, image::imageops::FilterType::Lanczos3);
        let rgb = resized.to_rgb8();

        let mut data = vec![0f32; 1 * 112 * 112 * 3];
        for (x, y, pixel) in rgb.enumerate_pixels() {
            let idx = (y as usize * 112 + x as usize) * 3;
            data[idx] = (pixel[0] as f32 - 127.5) / 127.5; // R
            data[idx + 1] = (pixel[1] as f32 - 127.5) / 127.5; // G
            data[idx + 2] = (pixel[2] as f32 - 127.5) / 127.5; // B
        }

        let array =
            Array4::from_shape_vec((1, 112, 112, 3), data).context("Failed to create array")?;
        let cow_array = ndarray::CowArray::from(array.view().into_dyn());
        let input_tensor = Value::from_array(self.session.allocator(), &cow_array)?;
        let outputs = self.session.run(vec![input_tensor])?;

        let output_tensor: ort::tensor::OrtOwnedTensor<f32, _> = outputs[0].try_extract()?;
        let embedding = output_tensor
            .view()
            .as_slice()
            .ok_or_else(|| anyhow::anyhow!("Failed to extract embedding slice"))?
            .to_vec();

        Ok(embedding)
    }
}
