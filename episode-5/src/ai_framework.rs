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
  Visible(VisibleEnvironment)
}


trait Sensing
{
  fn sense(&self, environment: Environment) -> Option<Vec<f32>>;
}


impl Sensing for VisionSensor
{
  fn sense(&self, environment: Environment) -> Option<Vec<f32>>
  {
    match environment
    {
      Environment::Visible(visible_env) => {
        if let Some(image) = visible_env.image_assets.get(&self.visual_sensor)
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


#[derive(Component, Debug)]
struct VisibleEnvironment
{
  image_assets: Res<'static, Assets<Image>>,
}


// Need mappings of sensor-env:
//
// Vision-Space
// Touch-Agent
