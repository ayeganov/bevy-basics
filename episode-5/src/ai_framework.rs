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


trait Sensing
{
  fn sense(&self, environment: Environment, images: Res<Assets<Image>>) -> Option<Vec<f32>>;
}


impl Sensing for VisionSensor
{
  fn sense(&self, environment: Environment, images: Res<Assets<Image>>) -> Option<Vec<f32>>
  {
    let row_number = 100;
    match environment
    {
      Environment::VisibleEnvironment =>
      {
        if let Some(image) = images.get(&self.visual_sensor)
        {
          let width = image.width() as usize;
          let start = (row_number * width) as usize;
          let end = start + width;
          let region_is_valid = start < image.data.len() && end <= image.data.len();

          if region_is_valid
          {
            let row_data = image.data[start..end].iter().map(|&b| b as f32).collect();
            return Some(row_data);
          }
        }
        None
      },
      _ => { return None; }
    }
  }
}


// Need mappings of sensor-env:
//
// Vision-Space
// Touch-Agent
