mod asset_loader;
mod asteroids;
mod camera;
mod collision_detection;
mod debug;
mod despawn;
mod health;
mod movement;
mod schedule;
mod spaceship;
mod state;
mod vision;
mod ai_agent;
mod ai_framework;

use bevy::prelude::*;

use gpu_copy::GpuToCpuCpyPlugin;

//use debug::DebugPlugin;
use asset_loader::AssetLoaderPlugin;
use asteroids::AsteroidPlugin;
use bevy_editor_pls::prelude::*;
use bevy_mod_picking::prelude::*;
use camera::CameraPlugin;
use collision_detection::CollisionDetectionPlugin;
use despawn::DespawnPlugin;
use movement::MovementPlugin;
use schedule::SchedulePlugin;
use spaceship::SpaceshipPlugin;
use state::StatePlugin;
use vision::VisionPlugin;
use ai_agent::AiAgentPlugin;


fn main()
{
  let (w, h) = (1280, 720);
  App::new()
    // Bevy built-ins.
    .insert_resource(ClearColor(Color::rgb(0.1, 0.0, 0.15)))
      .insert_resource(gpu_copy::SceneInfo::new(w, h))
    .insert_resource(AmbientLight {
      color: Color::default(),
      brightness: 0.75,
    })
    .add_plugins(DefaultPlugins)
    // User defined plugins.
    .add_plugins(AssetLoaderPlugin)
    .add_plugins(MovementPlugin)
    .add_plugins(SpaceshipPlugin)
    .add_plugins(AsteroidPlugin)
    .add_plugins(CameraPlugin)
    .add_plugins(CollisionDetectionPlugin)
    .add_plugins(DespawnPlugin)
    .add_plugins(SchedulePlugin)
    .add_plugins(StatePlugin)
    .add_plugins(DefaultPickingPlugins)
    .add_plugins(VisionPlugin)
    .add_plugins(AiAgentPlugin)
    .add_plugins(GpuToCpuCpyPlugin)
//    .add_plugins(EditorPlugin::default())
    // .add_plugins(DebugPlugin)
    .run();
}
