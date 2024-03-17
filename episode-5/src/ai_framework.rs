use bevy::prelude::*;

use crate::vision::Vision as VisionSensor;


//trait Environment: Any
//{
//  fn describe(&self) -> String;
//}


#[derive(Component, Debug, Clone)]
pub enum Sensors
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
    match environment
    {
      Environment::VisibleEnvironment => {
        if let Some(image) = images.get(&self.visual_sensor)
        {
          let data = image.data.iter().map(|&b| b as f32).collect();
          return Some(data);
        }
        None
      },
      _ => { return None; }
    }
  }
}


// An Agent, capable of employing any sensor that can sense an environment
struct Agent<S: Sensing>
{
  sensor: S,
}


// Need mappings of sensor-env:
//
// Vision-Space
// Touch-Agent
