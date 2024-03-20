use bevy::prelude::*;

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


impl Sensing for VisionSensor
{
  fn sense(&self, environment: Environment, images: &Res<Assets<Image>>) -> Option<Vec<f32>>
  {
    let row_number = 100;
    match environment
    {
      Environment::VisibleEnvironment =>
      {
        info!("sensing Image address: {:?}", &self.visual_sensor);
        if let Some(handle) = &self.visual_sensor
        {
          info!("Handle: {:?}", handle);
          if let Some(image) = images.get(handle)
          {
            image.texture_descriptor.label.as_ref().map(|label| info!("Label: {:?}", label));
            info!("Texture size: {:?}", (image.texture_descriptor.size.width, image.texture_descriptor.size.height));
            info!("Image size: {:?}", (image.size()));
            let width = image.width() as usize;
            let start = (row_number * width) as usize;
            let end = start + width;
            let region_is_valid = start < image.data.len() && end <= image.data.len();

            if region_is_valid
            {
              let row_data = image.data[start..end].iter().map(|&b| b as f32).collect();
              return Some(row_data);
            }
            else
            {
  //            println!("Invalid region for sensor: {:?}", self.visual_sensor);
            }
          }
          else {
            println!("No image found for sensor: {:?}", self.visual_sensor);
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
