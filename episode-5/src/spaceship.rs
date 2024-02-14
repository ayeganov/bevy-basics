use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::{
    prelude::*,
    render::{
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
//        camera::RenderTarget,
        camera::Viewport
    },
};

use bevy_mod_picking::prelude::*;


use crate::{
  asset_loader::SceneAssets,
  collision_detection::{Collider, CollisionDamage},
  health::Health,
  movement::{Acceleration, MovingObjectBundle, Velocity},
  schedule::InGameSet,
  state::GameState,
};


const STARTING_TRANSLATION: Vec3 = Vec3::new(0.0, 0.0, -20.0);
const SPACESHIP_RADIUS: f32 = 5.0;
const SPACESHIP_SPEED: f32 = 25.0;
const SPACESHIP_ROTATION_SPEED: f32 = 2.5;
const SPACESHIP_ROLL_SPEED: f32 = 2.5;
const SPACESHIP_HEALTH: f32 = 100.0;
const SPACESHIP_COLLISION_DAMAGE: f32 = 100.0;
const MISSILE_SPEED: f32 = 50.0;
const MISSILE_FORWARD_SPAWN_SCALAR: f32 = 7.5;
const MISSILE_RADIUS: f32 = 1.0;
const MISSILE_HEALTH: f32 = 1.0;
const MISSILE_COLLISION_DAMAGE: f32 = 5.0;

#[derive(Component, Debug)]
pub struct Spaceship;


#[derive(Component, Debug)]
pub struct SpaceshipShield;


#[derive(Component, Debug)]
pub struct SpaceshipMissile;


#[derive(Component, Debug)]
pub struct Vision
{
  handle: Handle<Image>
}


#[derive(Component)]
pub struct UnpickableGLTF;



pub struct SpaceshipPlugin;


impl Plugin for SpaceshipPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(PostStartup, (spawn_spaceship, make_spaceship_pickable.after(spawn_spaceship)))
        .add_systems(OnEnter(GameState::GameOver), spawn_spaceship)
        .add_systems(
            Update,
            (
                spaceship_movement_controls,
                spaceship_weapon_controls,
                spaceship_shield_controls,
                draw_selected_vision,
            )
                .chain()
                .in_set(InGameSet::UserInput),
        )
        .add_systems(Update, spaceship_destroyed.in_set(InGameSet::EntityUpdates));
  }
}


fn draw_selected_vision(mut gizmos: Gizmos,
                        query_spaceship: Query<(Entity, &Children, &PickSelection), With<Spaceship>>,
                        query_proj: Query<(&PerspectiveProjection, &GlobalTransform)>)
{
  for (_spaceship, children, pick_selection) in query_spaceship.iter()
  {
    if pick_selection.is_selected
    {
      info!("You got me buddy!");
      for &child in children.iter()
      {
        if let Ok((projection, &transform)) = query_proj.get(child)
        {
          let half_fov = projection.fov / 2.0;
          let tan_half_fov = half_fov.tan();
          let near_height = 2.0 * tan_half_fov * projection.near;
          let near_width = near_height * projection.aspect_ratio;
          let far_height = 2.0 * tan_half_fov * projection.far;
          let far_width = far_height * projection.aspect_ratio;

          // Near plane corners
          let near_top_left = transform * Vec3::new(-near_width / 2.0, near_height / 2.0, -projection.near);
          let near_top_right = transform * Vec3::new(near_width / 2.0, near_height / 2.0, -projection.near);
          let near_bottom_left = transform * Vec3::new(-near_width / 2.0, -near_height / 2.0, -projection.near);
          let near_bottom_right = transform * Vec3::new(near_width / 2.0, -near_height / 2.0, -projection.near);

          // Far plane corners
          let far_top_left = transform * Vec3::new(-far_width / 2.0, far_height / 2.0, -projection.far);
          let far_top_right = transform * Vec3::new(far_width / 2.0, far_height / 2.0, -projection.far);
          let far_bottom_left = transform * Vec3::new(-far_width / 2.0, -far_height / 2.0, -projection.far);
          let far_bottom_right = transform * Vec3::new(far_width / 2.0, -far_height / 2.0, -projection.far);

          // Draw lines between corners to form the frustum
          let color = Color::rgba(0.0, 1.0, 0.0, 0.5); // Green, semi-transparent

          // Near plane
          gizmos.line(near_top_left, near_top_right, color);
          gizmos.line(near_top_right, near_bottom_right, color);
          gizmos.line(near_bottom_right, near_bottom_left, color);
          gizmos.line(near_bottom_left, near_top_left, color);

          // Far plane
          gizmos.line(far_top_left, far_top_right, color);
          gizmos.line(far_top_right, far_bottom_right, color);
          gizmos.line(far_bottom_right, far_bottom_left, color);
          gizmos.line(far_bottom_left, far_top_left, color);

          // Edges between near and far planes
          gizmos.line(near_top_left, far_top_left, color);
          gizmos.line(near_top_right, far_top_right, color);
          gizmos.line(near_bottom_left, far_bottom_left, color);
          gizmos.line(near_bottom_right, far_bottom_right, color);
        }
      }
    }
  }
}


fn create_spaceship_vision() -> Image
{
  let size = Extent3d {
    width: 50,
    height: 200,
    ..default()
  };

  // This is the texture that will be rendered to.
  let mut image = Image {
    texture_descriptor: TextureDescriptor {
      label: None,
      size,
      dimension: TextureDimension::D2,
      format: TextureFormat::Bgra8UnormSrgb,
      mip_level_count: 1,
      sample_count: 1,
      usage: TextureUsages::TEXTURE_BINDING
          | TextureUsages::COPY_DST
          | TextureUsages::RENDER_ATTACHMENT,
      view_formats: &[],
    },
    ..default()
  };

  // fill image.data with zeroes
  image.resize(size);

  image
}


fn set_pickible_recursive(
  commands: &mut Commands,
  entity: &Entity,
  mesh_query: &Query<Entity, With<Handle<Mesh>>>,
  children_query: &Query<&Children>,
)
{
  for mesh_entity in mesh_query.iter()
  {
    info!("Yay!, we got it!");
    commands.entity(mesh_entity).insert(PickableBundle::default());
  }

  if let Ok(children) = children_query.get(*entity)
  {
    for child in children.iter()
    {
      set_pickible_recursive(commands, child, mesh_query, children_query);
    }
  }
}


fn make_spaceship_pickable(
  mut commands: Commands,
  mut unpickable_query: Query<(Entity, &Children), With<UnpickableGLTF>>,
  mesh_query: Query<(Entity), With<Handle<Mesh>>>,
  children_query: Query<&Children>
)
{
  for (entity, _children) in unpickable_query.iter_mut()
  {
    info!(" [MODELS] Setting Pickable on {:?}", entity);
    set_pickible_recursive(&mut commands, &entity, &mesh_query, &children_query);
    commands.entity(entity).remove::<UnpickableGLTF>();
  }
}


fn spawn_spaceship(mut commands: Commands, scene_assets: Res<SceneAssets>, mut images: ResMut<Assets<Image>>)
{
    let vision = create_spaceship_vision();
    let handle = images.add(vision);

    // TODO: use render target to read the camera viewed pixels
//   let vision_image_clone = handle.clone();
    let parent_id = commands.spawn((
        MovingObjectBundle {
            velocity: Velocity::new(Vec3::ZERO),
            acceleration: Acceleration::new(Vec3::ZERO),
            collider: Collider::new(SPACESHIP_RADIUS),
            model: SceneBundle {
                scene: scene_assets.spaceship.clone(),
                transform: Transform::from_translation(STARTING_TRANSLATION),
                ..default()
            },
        },
        Spaceship,
        UnpickableGLTF,
        Vision { handle },
        PickableBundle::default(),
        Health::new(SPACESHIP_HEALTH),
        CollisionDamage::new(SPACESHIP_COLLISION_DAMAGE),
    )).id();


    let camera_id = commands.spawn(Camera3dBundle {
        camera_3d: Camera3d {
            clear_color: ClearColorConfig::None,
            ..default()
        },
        camera: Camera {
          // render before the "main pass" camera
          order: 2,
//            target: RenderTarget::Image(vision_image_clone),
          viewport: Some(Viewport {
            physical_position: UVec2::new(0, 0),
            physical_size: UVec2::new(256, 256),
            ..default()
          }),
          ..default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 1.0, 8.0))
            .looking_at(Vec3::new(0.0, 1.0, 30.), -Vec3::Y),
        ..default()
    }).id();

    commands.entity(parent_id).push_children(&[camera_id]);
}


fn spaceship_movement_controls(
    mut query: Query<(&mut Transform, &mut Velocity), With<Spaceship>>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let Ok((mut transform, mut velocity)) = query.get_single_mut() else {
        return;
    };
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
    velocity.value = -transform.forward() * movement;
}


fn spaceship_weapon_controls(
    mut commands: Commands,
    query: Query<&Transform, With<Spaceship>>,
    keyboard_input: Res<Input<KeyCode>>,
    scene_assets: Res<SceneAssets>,
) {
    let Ok(transform) = query.get_single() else {
        return;
    };
    if keyboard_input.pressed(KeyCode::Space) {
        commands.spawn((
            MovingObjectBundle {
                velocity: Velocity::new(-transform.forward() * MISSILE_SPEED),
                acceleration: Acceleration::new(Vec3::ZERO),
                collider: Collider::new(MISSILE_RADIUS),
                model: SceneBundle {
                    scene: scene_assets.missiles.clone(),
                    transform: Transform::from_translation(
                        transform.translation + -transform.forward() * MISSILE_FORWARD_SPAWN_SCALAR,
                    ),
                    ..default()
                },
            },
            SpaceshipMissile,
            Health::new(MISSILE_HEALTH),
            CollisionDamage::new(MISSILE_COLLISION_DAMAGE),
        ));
    }
}

fn spaceship_shield_controls(
    mut commands: Commands,
    query: Query<Entity, With<Spaceship>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let Ok(spaceship) = query.get_single() else {
        return;
    };
    if keyboard_input.pressed(KeyCode::Tab) {
        commands.entity(spaceship).insert(SpaceshipShield);
    }
}

fn spaceship_destroyed(
    mut next_state: ResMut<NextState<GameState>>,
    query: Query<(), With<Spaceship>>,
)
{
  if query.get_single().is_err() {
    next_state.set(GameState::GameOver);
  };
}
