use bevy::{
  core_pipeline::clear_color::ClearColorConfig,
  prelude::*,
  math::vec4,
  render::{
    render_resource::{
      Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
    camera::Viewport
  }
};

use bevy_mod_picking::prelude::*;

use crate::schedule::InGameSet;

#[derive(Component, Debug)]
pub struct Vision
{
//  pub visual_sensor: Handle<Image>,
}


#[derive(Component, Debug)]
pub struct VisionSensing;


#[derive(Bundle)]
pub struct VisionObjectBundle
{
  pub vision: Vision,
}


impl Default for VisionObjectBundle
{
  fn default() -> Self
  {
    Self
    {
      vision: Vision { }
    }
  }
}


pub struct VisionPlugin;


impl Plugin for VisionPlugin
{
  fn build(&self, app: &mut App)
  {
    app.add_systems(
      Update,
      (make_pickable, )
        .chain()
        .in_set(InGameSet::EntityUpdates),
    )
    .add_systems(Update, handle_vision_selection.run_if(on_event::<VisionSelected>()))
    .add_event::<VisionSelected>();
  }
}


#[derive(Event)]
struct VisionSelected(Entity);

impl From<ListenerInput<Pointer<Click>>> for VisionSelected
{
  fn from(event: ListenerInput<Pointer<Click>>) -> Self
  {
    VisionSelected(event.listener())
  }
}


fn create_vision_sensor() -> Image
{
  let size = Extent3d {
    width: 50,
    height: 200,
    ..default()
  };

  // This is the texture that will be rendered to.
  let mut image = Image
  {
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


fn make_pickable(mut commands: Commands,
                 meshes: Query<Entity, (With<Handle<Mesh>>, Without<Pickable>)>,
)
{
  for entity in meshes.iter()
  {
    commands
      .entity(entity)
      .insert((PickableBundle::default(), HIGHLIGHT_TINT.clone()));
  }
}


const HIGHLIGHT_TINT: Highlight<StandardMaterial> = Highlight
{
  hovered: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
    base_color: matl.base_color + vec4(-0.5, -0.3, 0.9, 0.8), // hovered is blue
    ..matl.to_owned()
  })),

  pressed: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
    base_color: matl.base_color + vec4(-0.4, -0.4, 0.8, 0.8), // pressed is a different blue
    ..matl.to_owned()
  })),

  selected: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
    base_color: matl.base_color + vec4(-0.4, 0.8, -0.4, 0.0), // selected is green
    ..matl.to_owned()
  })),
};


fn attach_vision_camera(commands: &mut Commands,
                        vision_id: Entity)
{
  let camera_id = commands.spawn((Camera3dBundle
  {
    camera_3d: Camera3d
    {
      clear_color: ClearColorConfig::None,
      ..default()
    },
    camera: Camera
    {
      // render before the "main pass" camera
      order: 1,
//      target: RenderTarget::Image(vision_image_clone),
      viewport: Some(Viewport {
        physical_position: UVec2::new(0, 0),
        physical_size: UVec2::new(256, 256),
        ..default()
      }),
      ..default()
    },
    transform: Transform::from_translation(Vec3::new(0.0, -1.0, -7.0))
        .looking_at(Vec3::new(0.0, -1.0, -30.), Vec3::Y),
    projection: PerspectiveProjection
    {
      far: 500.0,
      ..default()
    }.into(),
    ..default()
  },
  )).id();

  commands.entity(vision_id).push_children(&[camera_id]);
}


fn detach_vision_camera(selected_vision: Entity,
                        commands: &mut Commands,
)
{
  commands.entity(selected_vision).remove::<Camera3dBundle>();
}


fn unselect_vision(selected_vision: Entity,
                      commands: &mut Commands,
)
{
  commands.entity(selected_vision).remove::<PickSelection>();
}


fn handle_vision_selection(mut selected: EventReader<VisionSelected>,
                           vision_query: Query<Entity, With<Vision>>,
                           already_selected_query: Query<Entity, (With<Vision>, With<PickSelection>)>,
                           mut commands: Commands,
)
{
  info!("You clicked!");
  if !already_selected_query.is_empty()
  {
    let selected_vision = already_selected_query.single();
    detach_vision_camera(selected_vision, &mut commands);
    unselect_vision(selected_vision, &mut commands);
  }

  for VisionSelected(selected_vision_id) in selected.read()
  {
    for vision_id in vision_query.iter()
    {
      if vision_id == *selected_vision_id
      {
        info!("Gotcha!");
        info!("Vision: {:?}", vision_id);
        commands.entity(vision_id).insert(PickSelection {
          is_selected: true
        });
        attach_vision_camera(&mut commands, vision_id);
      }
    }
  }
}


fn draw_selected_vision(mut gizmos: Gizmos,
                        query_spaceship: Query<(Entity, &Children, &PickSelection), (With<Vision>, With<PickSelection>)>,
                        pickables_query: Query<(Entity, &PickSelection), With<PickSelection>>,
                        query_proj: Query<(&Projection, &GlobalTransform)>)
{
//  for (pick_id, pick_selection) in pickables_query.iter()
//  {
//    if pick_selection.is_selected
//    {
//      info!("Selected pick id: {:?}", pick_id);
//  for (_spaceship, children) in query_spaceship.iter()
//  {
//    info!("Spaceship id: {:?}", _spaceship);
//    if true
//    {
//      for &child in children.iter()
//      {
//        if let Ok((projection, &transform)) = query_proj.get(child)
//        {
//          match projection
//          {
//            Projection::Perspective(proj) =>
//            {
//              let half_fov = proj.fov / 2.0;
//              let tan_half_fov = half_fov.tan();
//              let near_height = 2.0 * tan_half_fov * proj.near;
//              let near_width = near_height * proj.aspect_ratio;
//              let far_height = 2.0 * tan_half_fov * proj.far;
//              let far_width = far_height * proj.aspect_ratio;
//
//              // Near plane corners
//              let near_top_left = transform * Vec3::new(-near_width / 2.0, near_height / 2.0, -proj.near);
//              let near_top_right = transform * Vec3::new(near_width / 2.0, near_height / 2.0, -proj.near);
//              let near_bottom_left = transform * Vec3::new(-near_width / 2.0, -near_height / 2.0, -proj.near);
//              let near_bottom_right = transform * Vec3::new(near_width / 2.0, -near_height / 2.0, -proj.near);
//
//              // Far plane corners
//              let far_top_left = transform * Vec3::new(-far_width / 2.0, far_height / 2.0, -proj.far);
//              let far_top_right = transform * Vec3::new(far_width / 2.0, far_height / 2.0, -proj.far);
//              let far_bottom_left = transform * Vec3::new(-far_width / 2.0, -far_height / 2.0, -proj.far);
//              let far_bottom_right = transform * Vec3::new(far_width / 2.0, -far_height / 2.0, -proj.far);
//
//              // Draw lines between corners to form the frustum
//              let color = Color::rgba(0.0, 1.0, 0.0, 0.5); // Green, semi-transparent
//
//              // Near plane
//              gizmos.line(near_top_left, near_top_right, color);
//              gizmos.line(near_top_right, near_bottom_right, color);
//              gizmos.line(near_bottom_right, near_bottom_left, color);
//              gizmos.line(near_bottom_left, near_top_left, color);
//
//              // Far plane
//              gizmos.line(far_top_left, far_top_right, color);
//              gizmos.line(far_top_right, far_bottom_right, color);
//              gizmos.line(far_bottom_right, far_bottom_left, color);
//              gizmos.line(far_bottom_left, far_top_left, color);
//
//              // Edges between near and far planes
//              gizmos.line(near_top_left, far_top_left, color);
//              gizmos.line(near_top_right, far_top_right, color);
//              gizmos.line(near_bottom_left, far_bottom_left, color);
//              gizmos.line(near_bottom_right, far_bottom_right, color);
//            },
//            _ => {}
//          }
//        }
//      }
//    }
//  }
//    }
//  }

  for (_spaceship, children, pick) in query_spaceship.iter()
  {
    if pick.is_selected
    {
      info!("Spaceship id: {:?}", _spaceship);
      for &child in children.iter()
      {
        if let Ok((projection, &transform)) = query_proj.get(child)
        {
          match projection
          {
            Projection::Perspective(proj) =>
            {
              let half_fov = proj.fov / 2.0;
              let tan_half_fov = half_fov.tan();
              let near_height = 2.0 * tan_half_fov * proj.near;
              let near_width = near_height * proj.aspect_ratio;
              let far_height = 2.0 * tan_half_fov * proj.far;
              let far_width = far_height * proj.aspect_ratio;

              // Near plane corners
              let near_top_left = transform * Vec3::new(-near_width / 2.0, near_height / 2.0, -proj.near);
              let near_top_right = transform * Vec3::new(near_width / 2.0, near_height / 2.0, -proj.near);
              let near_bottom_left = transform * Vec3::new(-near_width / 2.0, -near_height / 2.0, -proj.near);
              let near_bottom_right = transform * Vec3::new(near_width / 2.0, -near_height / 2.0, -proj.near);

              // Far plane corners
              let far_top_left = transform * Vec3::new(-far_width / 2.0, far_height / 2.0, -proj.far);
              let far_top_right = transform * Vec3::new(far_width / 2.0, far_height / 2.0, -proj.far);
              let far_bottom_left = transform * Vec3::new(-far_width / 2.0, -far_height / 2.0, -proj.far);
              let far_bottom_right = transform * Vec3::new(far_width / 2.0, -far_height / 2.0, -proj.far);

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
            },
            _ => {}
          }
        }
      }
    }
  }
}
