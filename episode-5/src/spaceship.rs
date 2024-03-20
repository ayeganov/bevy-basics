use bevy::{prelude::*, transform};

use rand::prelude::*;


use crate::{
  asset_loader::SceneAssets,
  collision_detection::{Collider, CollisionDamage},
  camera::{VisibleRange, update_visible_range},
  health::Health,
  movement::{Acceleration, MovingObjectBundle, Velocity},
  vision::VisionObjectBundle,
  schedule::InGameSet,
  state::GameState,
  ai_agent::AiAgent
};


const SPACESHIP_RADIUS: f32 = 0.65;
const SPACESHIP_SPEED: f32 = 15.0;
const SPACESHIP_ROTATION_SPEED: f32 = 2.5;
const SPACESHIP_ROLL_SPEED: f32 = 2.5;
const SPACESHIP_HEALTH: f32 = 100.0;
const SPACESHIP_COLLISION_DAMAGE: f32 = 100.0;
const SPACESHIP_SCALE: Vec3 = Vec3::splat(0.2);
const MISSILE_SPEED: f32 = 50.0;
const MISSILE_FORWARD_SPAWN_SCALAR: f32 = 2.0;
const MISSILE_RADIUS: f32 = 0.3;
const MISSILE_HEALTH: f32 = 1.0;
const MISSILE_COLLISION_DAMAGE: f32 = 5.0;
const MISSILE_SCALE: Vec3 = Vec3::splat(0.3);
const NUM_SPACESHIPS: u16 = 1;


#[derive(Component, Debug)]
pub struct Spaceship;


#[derive(Component, Debug)]
pub struct SpaceshipShield;


#[derive(Component, Debug)]
pub struct SpaceshipMissile;


pub struct SpaceshipPlugin;


impl Plugin for SpaceshipPlugin
{
  fn build(&self, app: &mut App)
  {
    app.add_systems(PostStartup, spawn_spaceships.after(update_visible_range))
      .add_systems(OnEnter(GameState::GameOver), spawn_spaceships)
      .add_systems(
        Update,
        (
          spaceship_movement_controls,
          spaceship_weapon_controls,
          spaceship_shield_controls,
        )
        .chain()
        .in_set(InGameSet::UserInput),
      )
      .add_systems(Update, spaceship_destroyed.in_set(InGameSet::EntityUpdates));
  }
}


fn spawn_spaceships(mut commands: Commands,
                    scene_assets: Res<SceneAssets>,
                    visible_range: Res<VisibleRange>,
)
{
  let mut rng = rand::thread_rng();

  let id_offset = 2;
  for spaceship_num in 0..NUM_SPACESHIPS
  {
    let location = Vec3::new(
      rng.gen_range(visible_range.x_range.clone()),
      0.0, // Assuming asteroids move in the XZ plane, Y is set to 0 or another appropriate value
      rng.gen_range(visible_range.z_range.clone()),
    );

    spawn_spaceship(&mut commands, &scene_assets, location, spaceship_num + id_offset);
  }
}


fn spawn_spaceship(commands: &mut Commands,
                   scene_assets: &Res<SceneAssets>,
                   location: Vec3,
                   spaceship_num: u16
)
{
  commands.spawn((
    MovingObjectBundle {
      velocity: Velocity::new(Vec3::ZERO),
      acceleration: Acceleration::new(Vec3::ZERO),
      collider: Collider::new(SPACESHIP_RADIUS),
      model: SceneBundle
      {
        scene: scene_assets.spaceship.clone(),
        transform: Transform::from_translation(location)
                             .with_scale(SPACESHIP_SCALE),
        ..default()
      },
    },
    VisionObjectBundle::new(spaceship_num as isize),
    Spaceship,
    AiAgent,
    Health::new(SPACESHIP_HEALTH),
    CollisionDamage::new(SPACESHIP_COLLISION_DAMAGE),
  ));
}


fn spaceship_movement_controls(
    mut query: Query<(&mut Transform, &mut Velocity), With<Spaceship>>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
)
{
  let Ok((_transform, _velocity)) = query.get_single_mut() else {
      return;
  };
  for (mut transform, mut velocity) in query.iter_mut()
  {
    let mut rotation = 0.0;
    let mut roll = 0.0;
    let mut movement = 0.0;

    if keyboard_input.pressed(KeyCode::D) {
        rotation = -SPACESHIP_ROTATION_SPEED * time.delta_seconds();
    } else if keyboard_input.pressed(KeyCode::A) {
        rotation = SPACESHIP_ROTATION_SPEED * time.delta_seconds();
    }

    if keyboard_input.pressed(KeyCode::S) {
        movement = -SPACESHIP_SPEED;
    } else if keyboard_input.pressed(KeyCode::W) {
        movement = SPACESHIP_SPEED;
    }

    if keyboard_input.pressed(KeyCode::ShiftLeft) {
        roll = -SPACESHIP_ROLL_SPEED * time.delta_seconds();
    } else if keyboard_input.pressed(KeyCode::ControlLeft) {
        roll = SPACESHIP_ROLL_SPEED * time.delta_seconds();
    }

    // Rotate around the Y-axis.
    // Ignores the Z-axis rotation applied below.
    transform.rotate_y(rotation);

    // Rotate around the local Z-axis.
    // The rotation is relative to the current rotation!
    transform.rotate_local_z(roll);

    // Update the spaceship's velocity based on new direction.
    velocity.value = transform.forward() * movement;
  }
}


fn spaceship_weapon_controls(
    mut commands: Commands,
    query: Query<&Transform, With<Spaceship>>,
    keyboard_input: Res<Input<KeyCode>>,
    scene_assets: Res<SceneAssets>,
)
{
//  let Ok(transform) = query.get_single() else {
//    return;
//  };

  if keyboard_input.pressed(KeyCode::Space)
  {
    for transform in query.iter()
    {
      commands.spawn((
        MovingObjectBundle
        {
          velocity: Velocity::new(transform.forward() * MISSILE_SPEED),
          acceleration: Acceleration::new(Vec3::ZERO),
          collider: Collider::new(MISSILE_RADIUS),
          model: SceneBundle {
            scene: scene_assets.missiles.clone(),
            transform: Transform::from_translation(
              transform.translation + transform.forward() * MISSILE_FORWARD_SPAWN_SCALAR,
            ).with_scale(MISSILE_SCALE),
            ..default()
          },
        },
        SpaceshipMissile,
        Health::new(MISSILE_HEALTH),
        CollisionDamage::new(MISSILE_COLLISION_DAMAGE),
      ));
    }
  }
}


fn spaceship_shield_controls(
    mut commands: Commands,
    query: Query<Entity, With<Spaceship>>,
    keyboard_input: Res<Input<KeyCode>>,
)
{
  let Ok(spaceship) = query.get_single() else
  {
    return;
  };

  if keyboard_input.pressed(KeyCode::Tab)
  {
    commands.entity(spaceship).insert(SpaceshipShield);
  }
}


fn spaceship_destroyed(
    mut next_state: ResMut<NextState<GameState>>,
    query: Query<(), With<Spaceship>>,
)
{
//  if query.get_single().is_err()
//  {
//    info!("Game Over!");
//    next_state.set(GameState::GameOver);
//  };
}
