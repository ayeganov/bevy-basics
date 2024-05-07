use bevy::prelude::*;

use crate::{ai_agent::{Agent, ShootEvent}, asset_loader::SceneAssets, collision_detection::{Collider, CollisionDamage}, health::Health, movement::{Acceleration, MovingObjectBundle, Velocity}};


pub struct EventHandlerPlugin;

#[derive(Component, Debug)]
pub struct SpaceshipMissile;

const MISSILE_SPEED: f32 = 50.0;
const MISSILE_FORWARD_SPAWN_SCALAR: f32 = 2.0;
const MISSILE_RADIUS: f32 = 0.3;
const MISSILE_HEALTH: f32 = 1.0;
const MISSILE_COLLISION_DAMAGE: f32 = 5.0;
const MISSILE_SCALE: Vec3 = Vec3::splat(0.3);


impl Plugin for EventHandlerPlugin
{
  fn build(&self, app: &mut App)
  {
    app.add_systems(Update, handle_shoot_events);
  }
}



fn handle_shoot_events(mut commands: Commands,
                       query: Query<&Transform, With<Agent>>,
                       scene_assets: Res<SceneAssets>,
                       mut shooting_event_reader: EventReader<ShootEvent>,
)
{
  for &ShootEvent {
    entity
  } in shooting_event_reader.read()
  {
    if let Ok(transform) = query.get(entity)
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
