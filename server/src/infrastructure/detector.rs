use anyhow::Result;
use image::DynamicImage;
use rust_faces::{BlazeFaceParams, FaceDetection, FaceDetectorBuilder, InferParams, Provider, ToArray3};

use crate::domain::models::BoundingBox;

pub struct FaceDetector {
    detector: Box<dyn rust_faces::FaceDetector>,
}

impl FaceDetector {
    pub fn new(_model_path: &str) -> Result<Self> {
        let detector = FaceDetectorBuilder::new(FaceDetection::BlazeFace640(BlazeFaceParams {
            score_threshold: 0.5,
            ..Default::default()
        }))
            .download()
            .infer_params(InferParams {
                provider: Provider::OrtCpu,
                intra_threads: Some(5),
                ..Default::default()
            })
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build face detector: {:?}", e))?;

        Ok(Self { detector })
    }

    pub fn detect(&self, img: &DynamicImage) -> Vec<(DynamicImage, BoundingBox, f64)> {
        let rgb_img = img.to_rgb8();
        let arr = rgb_img.into_array3();
        let faces = self.detector.detect(arr.view().into_dyn()).unwrap_or_default();

        faces.into_iter().map(|face| {
            let rect = face.rect;
            let bbox = BoundingBox {
                x: rect.x as i32,
                y: rect.y as i32,
                width: rect.width as i32,
                height: rect.height as i32,
            };
            let cropped = crop_face(img, &bbox);
            (cropped, bbox, face.confidence as f64)
        }).collect()
    }
}

fn crop_face(img: &DynamicImage, bbox: &BoundingBox) -> DynamicImage {
    let x = bbox.x.max(0) as u32;
    let y = bbox.y.max(0) as u32;
    let w = bbox.width as u32;
    let h = bbox.height as u32;
    img.crop_imm(x, y, w, h)
}
