use bevy::{
    prelude::*,
    app::{App as Engine, ScheduleRunnerPlugin, Startup, Update},
    asset::Assets,
    core_pipeline::{clear_color::ClearColor, core_3d::Camera3dBundle, tonemapping::Tonemapping},
    ecs::system::{Commands, Res, ResMut},
    math::Vec3,
    render::{camera::{Camera, RenderTarget}, color::Color, texture::Image},
    transform::components::Transform, window::PrimaryWindow,
};
use bevy_headless::{CurrImageContainer, HeadlessPlugin, ImageSource};


fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut scene_controller: ResMut<bevy_headless::SceneInfo>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut export_sources: ResMut<Assets<ImageSource>>,
    windows: Query<&PrimaryWindow>,
) {
    let render_target = bevy_headless::setup_render_target(
      &mut commands,
      &mut images,
      &mut scene_controller,
    );

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


fn save_img(curr_img: Res<CurrImageContainer>)
{
  let curr_img = curr_img.0.lock();
  if !curr_img.extension.is_empty()
  {
    let path = curr_img.create_path("out");
    log::info!("path is {path}");
    let img = curr_img.img_buffer.clone();

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
      .filter_module("bevy_headless", log::LevelFilter::Info)
      .init();

  let (w, h) = (1280, 720);

  Engine::new()
      .insert_resource(bevy_headless::SceneInfo::new(w, h))
      .insert_resource(ClearColor(Color::rgb_u8(0, 0, 0)))
//      .add_plugins(DefaultPlugins)
      .add_plugins((
          HeadlessPlugin,
          ScheduleRunnerPlugin::run_loop(std::time::Duration::from_secs_f64(1.0 / 30.0)),
      ))
      .add_systems(Startup, setup)
      .add_systems(Update, save_img)
      .run();
}
