use std::ops::Range;

use bevy::{prelude::*, window::WindowResized};

pub const CAMERA_DISTANCE: f32 = 80.0;

#[derive(Component, Debug)]
pub struct MainCamera;

pub struct CameraPlugin;

#[derive(Resource, Debug, Default)]
pub struct VisibleRange
{
  pub x_range: Range<f32>,
  pub z_range: Range<f32>
}


impl Plugin for CameraPlugin
{
  fn build(&self, app: &mut App)
  {
    app.init_resource::<VisibleRange>()
       .add_systems(Startup, spawn_camera)
       .add_event::<WindowResized>()
       .add_systems(PostStartup, update_visible_range)
       .add_systems(PreUpdate, update_visible_range.run_if(on_event::<WindowResized>()));
  }
}

fn spawn_camera(mut commands: Commands)
{
  commands.spawn((
    Camera3dBundle
    {
      transform: Transform::from_xyz(0.0, CAMERA_DISTANCE, 0.0)
          .looking_at(Vec3::ZERO, Vec3::Z),
      ..default()
    },
    MainCamera,
  ));
}


pub fn update_visible_range(window_query: Query<&Window>,
                            camera_query: Query<&Projection, With<MainCamera>>,
                            mut visible_range: ResMut<VisibleRange>,
)
{
  info!("Window has been resized!");

  for window in window_query.iter()
  {
    if window.title == "Vision" { continue; };

    let aspect_ratio = window.width() as f32 / window.height() as f32;

    if let Ok(projection) = camera_query.get_single()
    {
      if let Projection::Perspective(perspective_projection) = projection
      {
        let fov = perspective_projection.fov;
        let visible_height = 2.0 * (CAMERA_DISTANCE * (fov / 2.0).tan());
        let visible_width = visible_height * aspect_ratio;

        // Calculate spawn ranges based on the visible area
        visible_range.x_range = (-visible_width / 2.0) .. (visible_width / 2.0);
        visible_range.z_range = (-visible_height / 2.0) .. (visible_height / 2.0);
        info!("visible range: {:?}", visible_range);
      }
    }
    else
    {
      visible_range.x_range = -30.0 .. 30.0;
      visible_range.z_range = -30.0 .. 30.0;
    }
  }
}
