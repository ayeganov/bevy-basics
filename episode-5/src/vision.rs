use std::marker::PhantomData;

use bevy::{
  core_pipeline::clear_color::{ClearColorConfig, self},
  prelude::*,
  math::vec4,
  render::{
    render_resource::{
      Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
    renderer::RenderDevice,
    camera::Viewport,
    view::RenderLayers
  },
  ecs::system::SystemParam
};

use bevy_mod_picking::prelude::*;

use crate::schedule::InGameSet;
use crate::ai_framework::Sensor;
use crate::gpu_copy::image_copy::ImageCopier;

use bevy_headless::{ImageSource, ExportImage, ExportedImages};
use image::{SubImage, RgbaImage, GenericImageView, ImageBuffer, Rgba};


#[derive(Debug, Default, Clone)]
pub struct ViewParams
{
  pub x: u32,
  pub y: u32,
  pub width: u32,
  pub height: u32,
}


#[derive(SystemParam)]
pub struct VisionView<'w, 's>
{
  exported_images: Res<'w, ExportedImages>,
  marker: PhantomData<&'s ()>,
}


impl<'w, 's> VisionView<'w, 's>
{
  pub fn get_view(&self, params: &ViewParams) -> ImageBuffer<Rgba<u8>, Vec<u8>>
  {
    let locked_images = self.exported_images.0.lock();
//    info!("Get view exported images. It has {} images. Address of the container: {:?}", locked_images.len(), locked_images.as_ptr() as *const Vec<ExportImage>);
    // TODO: must refactor the 0 index to actually fetch the correct image
    let image = &locked_images[0].0.read();
//    info!("Params for view: {:?}", params);
    image.view(params.x, params.y, params.width, params.height).to_image()
  }
}


#[derive(Component, Debug, Default, Clone)]
pub struct Vision
{
  pub id: isize,
  pub cam_id: Option<Entity>,
  pub selected_cam_id: Option<Entity>,
  pub visual_sensor: Option<ViewParams>,
}


#[derive(Component, Debug)]
pub struct VisionSensing;


#[derive(Component, Debug)]
pub struct VisionCam;


#[derive(Bundle)]
pub struct VisionObjectBundle
{
  vision: Sensor,
  pub click_event: On::<Pointer<Click>>
}


impl Default for VisionObjectBundle
{
  fn default() -> Self
  {
    Self
    {
      vision: Sensor::Vision(Vision::default()),
      click_event: On::<Pointer<Click>>::send_event::<VisionSelected>(),
    }
  }
}


impl VisionObjectBundle
{
  pub fn new(id: isize) -> Self
  {
    let mut default = VisionObjectBundle::default();
    match default.vision
    {
      Sensor::Vision(ref mut vision) =>
      {
        vision.id = id;
      },
      _ => {}
    }
    default
  }
}


pub struct VisionPlugin;


impl Plugin for VisionPlugin
{
  fn build(&self, app: &mut App)
  {
    app.add_systems(
      Update,
      (make_pickable, draw_selected_vision, add_vision)
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


fn add_vision(mut images: ResMut<Assets<Image>>,
              mut visions: Query<(Entity, &mut Sensor), (With<Sensor>, Without<VisionSensing>)>,
              mut commands: Commands,
              mut export_sources: ResMut<Assets<ImageSource>>,
              mut exported_images: ResMut<ExportedImages>,
)
{
  if visions.is_empty()
  {
    return;
  }

  let viewport_size = (50, 20);
  let (render_target, destination, viewports) = bevy_headless::setup_render_target(
    &mut commands,
    &mut images,
    &mut export_sources,
    &mut exported_images,
    viewport_size,
    visions.iter().count() as u32,
  );

  let mut clear_color = Some(ClearColorConfig::Custom(Color::rgb(0.0, 0.0, 0.0)));
  for ((vision_id, mut sensor), viewport) in visions.iter_mut().zip(viewports.iter())
  {
    match *sensor
    {
      Sensor::Vision(ref mut vision) =>
      {
        info!("Adding vision to id: {}", vision.id);

        vision.visual_sensor = Some(ViewParams
        {
          x: viewport.0,
          y: viewport.1,
          width: viewport_size.0,
          height: viewport_size.1,
        });

        let current_cc = match clear_color.take()
        {
          Some(cc) => cc,
          None => ClearColorConfig::None
        };

        let camera_id = commands.spawn((Camera3dBundle
        {
          camera_3d: Camera3d
          {
            clear_color: current_cc,
            ..default()
          },
          camera: Camera
          {
            // render before the "main pass" camera
            order: vision.id,
            target: render_target.clone(),
            viewport: Some(Viewport {
              physical_position: UVec2::new(viewport.0, viewport.1),
              physical_size: UVec2::new(viewport_size.0, viewport_size.1),
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

        vision.cam_id = Some(camera_id);

        commands.entity(camera_id).insert(VisionCam{});
        commands.entity(vision_id).push_children(&[camera_id]);
        commands.entity(vision_id).insert(VisionSensing{});
      }
      _ => {}
    }
  }
}


fn create_vision_sensor(commands: &mut Commands, render_device: &Res<RenderDevice>, images: &mut ResMut<Assets<Image>>) -> (Handle<Image>, Handle<Image>)
{
  let size = Extent3d {
    width: 50,
    height: 20,
    ..default()
  };

  let mut render_target_image = Image
  {
    texture_descriptor: TextureDescriptor {
      label: Some("Vision Source"),
      size,
      dimension: TextureDimension::D2,
      format: TextureFormat::Rgba8UnormSrgb,
      mip_level_count: 1,
      sample_count: 1,
      usage: TextureUsages::TEXTURE_BINDING
          | TextureUsages::COPY_DST
          | TextureUsages::COPY_SRC
          | TextureUsages::RENDER_ATTACHMENT,
      view_formats: &[],
    },
    ..default()
  };

  render_target_image.resize(size);
  let render_target_image_handle = images.add(render_target_image);

  let mut cpu_image = Image
  {
    texture_descriptor: TextureDescriptor {
      label: Some("Vision Destination"),
      size,
      dimension: TextureDimension::D2,
      format: TextureFormat::Rgba8UnormSrgb,
      mip_level_count: 1,
      sample_count: 1,
      usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
      view_formats: &[],
    },
    ..Default::default()
  };
  cpu_image.resize(size);

  let cpu_image_handle = images.add(cpu_image);

  commands.spawn(ImageCopier::new(
    render_target_image_handle.clone(),
    cpu_image_handle.clone(),
    size,
    render_device
  ));


  (render_target_image_handle, cpu_image_handle)
}


fn make_pickable(mut commands: Commands,
                 meshes: Query<Entity, (With<Handle<Mesh>>, Without<Pickable>)>,
)
{
  for entity in meshes.iter()
  {
    commands
      .entity(entity)
      .insert((PickableBundle::default(), HIGHLIGHT_TINT.clone()))
      .insert(RenderLayers::all());
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
                        vision_id: Entity,
                        vision: &Vision) -> Entity
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
      order: vision.id,
//      target: RenderTarget::Image(vision.visual_sensor.clone()),
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

  camera_id
}


fn detach_vision_camera(selected_cam: Option<Entity>,
                        commands: &mut Commands,
)
{
  if let Some(cam_id) = selected_cam
  {
    commands.entity(cam_id).despawn_recursive();
  }
}


fn unselect_vision(selected_vision: Entity,
                      commands: &mut Commands,
)
{
  commands.entity(selected_vision).remove::<PickSelection>();
}


fn handle_vision_selection(mut selected: EventReader<VisionSelected>,
                           mut params: ParamSet<(
                               Query<(Entity, &mut Vision), With<Vision>>,
                               Query<(Entity, &Vision), (With<Vision>, With<PickSelection>)>
                           )>,
                           mut commands: Commands,
)
{
  {
    let already_selected_query = params.p1();

    if !already_selected_query.is_empty()
    {
      let (selected_vision, vision) = already_selected_query.single();
      detach_vision_camera(vision.selected_cam_id, &mut commands);
      unselect_vision(selected_vision, &mut commands);
    }
  }

  for VisionSelected(selected_vision_id) in selected.read()
  {
    let mut vision_query = params.p0();
    for (vision_id, mut vision) in vision_query.iter_mut()
    {
      if vision_id == *selected_vision_id
      {
        commands.entity(vision_id).insert(PickSelection {
          is_selected: true
        });

        vision.selected_cam_id = Some(attach_vision_camera(&mut commands, vision_id, &vision));
        return;
      }
    }
  }
}


fn draw_selected_vision(mut gizmos: Gizmos,
                        query_vision: Query<(Entity, &Children, &PickSelection), (With<Vision>, With<PickSelection>)>,
                        query_proj: Query<(&Projection, &GlobalTransform), Without<VisionCam>>)
{
  for (_vision, children, pick) in query_vision.iter()
  {
    if pick.is_selected
    {
      for (_idx, &child) in children.iter().enumerate()
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
