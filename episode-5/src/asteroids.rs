use std::ops::Range;

use bevy::prelude::*;
use rand::prelude::*;

use bevy::window::Window;

use crate::{
    asset_loader::SceneAssets,
    collision_detection::{Collider, CollisionDamage},
    health::Health,
    movement::{Acceleration, MovingObjectBundle, Velocity},
    schedule::InGameSet,
    camera::{MainCamera, CAMERA_DISTANCE}
};

const VELOCITY_SCALAR: f32 = 5.0;
const ACCELERATION_SCALAR: f32 = 1.0;
const SPAWN_TIME_SECONDS: f32 = 1.0;
const ROTATE_SPEED: f32 = 2.5;
const RADIUS: f32 = 2.5;
const HEALTH: f32 = 80.0;
const COLLISION_DAMAGE: f32 = 35.0;

#[derive(Component, Debug)]
pub struct Asteroid;

#[derive(Resource, Debug)]
pub struct SpawnTimer {
    timer: Timer,
}

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin
{
  fn build(&self, app: &mut App)
  {
    app.insert_resource(SpawnTimer
    {
      timer: Timer::from_seconds(SPAWN_TIME_SECONDS, TimerMode::Repeating),
    })
    .add_systems(
      Update,
      (spawn_asteroid, rotate_asteroids).in_set(InGameSet::EntityUpdates),
    );
  }
}


fn calculate_spawn_ranges(
    window_query: Query<&Window>,
    camera_query: Query<&Projection, With<MainCamera>>,
) -> (Range<f32>, Range<f32>)
{
  // Attempt to get the primary window and camera components
  let window = window_query.single();
  let aspect_ratio = window.width() as f32 / window.height() as f32;

  if let Ok(projection) = camera_query.get_single()
  {
    if let Projection::Perspective(perspective_projection) = projection
    {
      let fov = perspective_projection.fov;
      let visible_height = 2.0 * (CAMERA_DISTANCE * (fov / 2.0).tan());
      let visible_width = visible_height * aspect_ratio;

      // Calculate spawn ranges based on the visible area
      let spawn_range_x: Range<f32> = (-visible_width / 2.0) .. (visible_width / 2.0);
      let spawn_range_z: Range<f32> = (-visible_height / 2.0) .. (visible_height / 2.0);

      return (
        spawn_range_x,
        spawn_range_z
      );
    }
  }

  // Default or error case
  (0.0..0.0, 0.0..0.0)
}


fn make_velocity_toward_screen(x_range: &Range<f32>,
                               z_range: &Range<f32>,
                               translation: Vec3) -> Vec3
{
  let screen_center = Vec3::new(
    (x_range.start + x_range.end) / 2.0,
    0.0,
    (z_range.start + z_range.end) / 2.0,
  );

  // Modify the velocity vector calculation
  let direction_to_center = (screen_center - translation).normalize_or_zero();

  // Ensure the asteroids are always flying towards the center or across the screen
  let velocity = direction_to_center * VELOCITY_SCALAR;

  velocity
}


fn spawn_asteroid(
  mut commands: Commands,
  mut spawn_timer: ResMut<SpawnTimer>,
  time: Res<Time>,
  scene_assets: Res<SceneAssets>,
  window_query: Query<&Window>,
  camera_query: Query<&Projection, With<MainCamera>>,
)
{
  spawn_timer.timer.tick(time.delta());
  if !spawn_timer.timer.just_finished() {
      return;
  }

  let (x_range, z_range) = calculate_spawn_ranges(window_query, camera_query);
  info!("x range: {:?}, z range: {:?}", x_range, z_range);

  let mut rng = rand::thread_rng();

  let spawn_edge = rng.gen_bool(0.5); // true for X edge, false for Z edge

  let translation = if spawn_edge
  {
    Vec3::new(
      if rng.gen_bool(0.5) { x_range.start } else { x_range.end },
      0.0, // Assuming asteroids move in the XZ plane, Y is set to 0 or another appropriate value
      rng.gen_range(z_range.start..=z_range.end),
    )
  }
  else
  {
    // Spawn on the Z edge
    Vec3::new(
      rng.gen_range(x_range.start..=x_range.end),
      0.0, // Assuming asteroids move in the XZ plane, Y is set to 0 or another appropriate value
      if rng.gen_bool(0.5) { z_range.start } else { z_range.end },
    )
  };


  let mut random_unit_vector =
      || Vec3::new(rng.gen_range(-1.0..1.0), 0., rng.gen_range(-1.0..1.0)).normalize_or_zero();
  let velocity = make_velocity_toward_screen(&x_range, &z_range, translation);
  let acceleration = random_unit_vector() * ACCELERATION_SCALAR;

  commands.spawn((
    MovingObjectBundle {
      acceleration: Acceleration::new(acceleration),
      velocity: Velocity::new(velocity),
      collider: Collider::new(RADIUS),
      model: SceneBundle
      {
        scene: scene_assets.asteroid.clone(),
        transform: Transform::from_translation(translation)
                             .with_scale(Vec3::splat(0.5)),
        ..default()
      },
    },
    Asteroid,
    Health::new(HEALTH),
    CollisionDamage::new(COLLISION_DAMAGE),
  ));
}


fn rotate_asteroids(mut query: Query<&mut Transform, With<Asteroid>>, time: Res<Time>)
{
  for mut transform in query.iter_mut()
  {
    transform.rotate_local_z(ROTATE_SPEED * time.delta_seconds());
  }
}
