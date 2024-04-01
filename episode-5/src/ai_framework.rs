use bevy::prelude::*;
use bevy::render::texture::Image;
use image::{ImageBuffer, Rgba};
use std::{path::Path, ops::Deref};

use rand::prelude::*;

use crate::vision::Vision as VisionSensor;


/// Sensors provide the limitations on what agents are able to interact with.
#[derive(Component, Debug, Clone)]
pub enum Sensor
{
  Vision(VisionSensor),
}


#[derive(Component, Debug)]
pub enum Environment
{
  VisibleEnvironment
}


pub trait Sensing
{
  fn sense(&self, environment: Environment, images: &Res<Assets<Image>>) -> Option<Vec<f32>>;
}


fn save_image_to_disk(image: &ImageBuffer<Rgba<u8>, Vec<u8>>, path: &Path) -> Result<(), image::ImageError>
{
  // Get the image dimensions
  let (width, height) = image.dimensions();

  if image.len() == (width * height * 4) as usize
  {
//    let data = image.data.deref();
//
//    // This function assumes the image is in RGBA8 format.
//    // If it's in a different format, conversion will be needed.
//    let image_buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, data)
//        .ok_or(image::ImageError::Decoding(image::error::DecodingError::new(image::error::ImageFormatHint::Unknown, "Failed to create image buffer from raw data")))?;

    info!("Address of image buffer: {:?}", image.as_ptr());
    image.save(path)
  }
  else
  {
    info!("Image width: {:?}", width);
    info!("Image height: {:?}", height);
    info!("Image data length: {:?}", image.len());
    Err(image::ImageError::Parameter(image::error::ParameterError::from_kind(
        image::error::ParameterErrorKind::DimensionMismatch,
    )))
  }

}


impl Sensing for VisionSensor
{
  fn sense(&self, environment: Environment, images: &Res<Assets<Image>>) -> Option<Vec<f32>>
  {
    let mut rng = rand::thread_rng();

    let row_number = 25;
    match environment
    {
      Environment::VisibleEnvironment =>
      {
        if let Some(image) = &self.visual_sensor
        {
          let frame_id = rng.next_u32();
          let filename = format!("/tmp/{}/ai_agent_{}.png", self.id, frame_id);
          let path = Path::new(filename.as_str());

          let buffer = image.0.read();
          match save_image_to_disk(&buffer, path)
          {
            Ok(_) => info!("Image saved to disk"),
            Err(e) => error!("Error saving image to disk: {:?}", e),
          }

//            info!("image data: {:?}", image.data);

//            image.texture_descriptor.label.as_ref().map(|label| info!("Label: {:?}", label));
          let width = buffer.width() as usize;
          let start = (row_number * width) as usize;
          let end = start + width;
          let region_is_valid = start < buffer.len() && end <= buffer.len();

          if region_is_valid
          {
            let row_data = buffer.as_raw()[start..end].iter().map(|&b| b as f32).collect();
            return Some(row_data);
          }
          else
          {
//            println!("Invalid region for sensor: {:?}", self.visual_sensor);
          }
          None
        }
        else
        {
          println!("No handle found for sensor: {:?}", self.visual_sensor);
          None
        }
      },
    }
  }
}


// Need mappings of sensor-env:
//
// Vision-Space
// Touch-Agent
