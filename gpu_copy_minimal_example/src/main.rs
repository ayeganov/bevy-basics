use bevy::{
    prelude::*,
    app::{App as Engine, ScheduleRunnerPlugin, Startup, Update},
    asset::Assets,
    core_pipeline::{clear_color::ClearColor, core_3d::Camera3dBundle, tonemapping::Tonemapping},
    ecs::system::{Commands, Res, ResMut},
    math::Vec3,
    render::{camera::{Camera, RenderTarget}, color::Color, texture::Image},
    transform::components::Transform
};
use gpu_copy::{setup_render_target, ImageSource, GpuToCpuCpyPlugin, ExportedImages};


fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut export_sources: ResMut<Assets<ImageSource>>,
    mut exported_images: ResMut<ExportedImages>,
) {
    let viewport_size = (1280, 720);
    let (render_target, _) = setup_render_target(
      &"minimal_example".to_string(),
      &mut commands,
      &mut images,
      &mut export_sources,
      &mut exported_images,
      viewport_size,
      1
    );

    match std::fs::create_dir("out")
    {
      Ok(_) => {}
      Err(e) => log::error!("Couldn't create directory | {e:?}"),
    }

    // circular base
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Circle::new(4.0).into()),
        material: materials.add(Color::WHITE.into()),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb_u8(124, 144, 255).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        tonemapping: Tonemapping::None,
        camera: Camera { target: render_target, ..default() },
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        camera: Camera { target: RenderTarget::Window(bevy::window::WindowRef::Primary), ..default() },
        ..default()
    });
}


fn save_img(exported_images: Res<ExportedImages>,
)
{
  let locked_images = exported_images.0.lock();
  if let Some(image) = &locked_images.get(&"minimal_example".to_string())
  {
    let image = &image.0.read();
    let path = format!("out/minimal_example_{}.png", image.frame_id);
    log::info!("path is {path}");
    let img = image.img_buffer.clone();

    std::thread::spawn(move ||
    {
      if let Err(e) = img.save(path)
      {
        log::error!("Couldn't save image | {e:?}");
      };
    });
  }
}


pub fn main()
{
  pretty_env_logger::formatted_builder()
      .filter_module("minimal", log::LevelFilter::Info)
      .filter_module("bevy", log::LevelFilter::Info)
      .filter_module("gpu_copy", log::LevelFilter::Info)
      .init();

  let (w, h) = (1280, 720);

  Engine::new()
      .insert_resource(gpu_copy::SceneInfo::new(w, h))
      .insert_resource(ClearColor(Color::rgb_u8(0, 0, 0)))
      .add_plugins(DefaultPlugins)
      .add_plugins((
          GpuToCpuCpyPlugin,
          ScheduleRunnerPlugin::run_loop(std::time::Duration::from_secs_f64(1.0 / 30.0)),
      ))
      .add_systems(Startup, setup)
      .add_systems(Update, save_img)
      .run();
}
