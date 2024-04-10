use std::sync::Arc;

use crate::{node::{ImageExportNode, NodeName}, utils::ImageWrapper};
use bevy::{
    app::{App, Plugin, PostUpdate},
    asset::{Asset, AssetApp, Handle},
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        query::{QueryItem, With, Without},
        schedule::{apply_deferred, IntoSystemConfigs, IntoSystemSetConfigs, SystemSet},
        system::{
            lifetimeless::SRes, Commands, Local, Query, Res, ResMut, Resource, SystemParamItem,
        },
    },
    reflect::Reflect,
    render::{
        camera::CameraUpdateSystem,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        graph::CameraDriverLabel,
        render_asset::{PrepareAssetError, RenderAsset, RenderAssetPlugin, RenderAssets, RenderAssetUsages},
        render_graph::RenderGraph,
        render_resource::{Buffer, BufferDescriptor, BufferUsages, Extent3d, MapMode},
        renderer::RenderDevice,
        texture::Image, Render, RenderApp, RenderSet
    }, utils::HashMap,
};
use futures::channel::oneshot;

use parking_lot::{Mutex, RwLock};
use wgpu::Maintain;
use ImageExportSystems::{SetupImageExport, SetupImageExportFlush};


#[derive(Asset, Clone, Default, Reflect)]
pub struct ImageSource(pub Handle<Image>);


#[derive(Clone, Default, Debug)]
pub struct ExportImage(pub Arc<RwLock<ImageWrapper>>);


impl ExportImage
{
  pub fn new(size: Extent3d) -> Self
  {
    Self(Arc::new(RwLock::new(ImageWrapper::new(size))))
  }
}


#[derive(Clone, Default, Resource)]
pub struct ExportedImages(pub Arc<Mutex<HashMap<String, ExportImage>>>);


impl From<Handle<Image>> for ImageSource
{
  fn from(value: Handle<Image>) -> Self
  {
    Self(value)
  }
}


#[derive(Component, Clone)]
pub struct ImageExportSettings
{
  pub name: String,
}


impl ImageExportSettings
{
  pub fn new(name: String) -> Self
  {
    Self { name }
  }
}


#[derive(Clone)]
pub struct GpuImageExport
{
  pub buffer: Buffer,
  pub source_handle: Handle<Image>,
  pub source_size: Extent3d,
  pub bytes_per_row: u32,
  pub padded_bytes_per_row: u32,
}


impl GpuImageExport {
  fn get_bps(&self) -> (usize, usize, Extent3d)
  {
    (self.bytes_per_row as usize, self.padded_bytes_per_row as usize, self.source_size)
  }
}


impl RenderAsset for ImageSource
{
  type Param = (SRes<RenderDevice>, SRes<RenderAssets<Image>>);
  type PreparedAsset = GpuImageExport;

  fn prepare_asset(
    self: Self,
    (device, images): &mut SystemParamItem<Self::Param>,
  ) -> Result<Self::PreparedAsset, PrepareAssetError<Self>>
  {
    let gpu_image = images.get(&self.0).unwrap();

    let size = gpu_image.texture.size();
    let format = &gpu_image.texture_format;
    let bytes_per_row = (size.width / format.block_dimensions().0) * format.block_copy_size(None).unwrap();

    let padded_bytes_per_row = RenderDevice::align_copy_bytes_per_row(bytes_per_row as usize) as u32;

    let source_size = gpu_image.texture.size();

    Ok(GpuImageExport
      {
        buffer: device.create_buffer(&BufferDescriptor {
          label: Some("Image Export Buffer"),
          size: (source_size.height * padded_bytes_per_row) as u64,
          usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
          mapped_at_creation: false,
        }),
        source_handle: self.0.clone(),
        source_size,
        bytes_per_row,
        padded_bytes_per_row,
    })
  }

  fn asset_usage(&self) -> RenderAssetUsages
  {
    RenderAssetUsages::default()
  }
}


#[derive(Component, Clone)]
pub struct ImageExportStartFrame(u64);

impl Default for ImageExportSettings
{
  fn default() -> Self
  {
    Self { name: "default_export".into() }
  }
}


impl ExtractComponent for ImageExportSettings
{
  type QueryFilter = ();
  type Out = (Self, Handle<ImageSource>, ImageExportStartFrame);
  type QueryData =
      (&'static Self, &'static Handle<ImageSource>, &'static ImageExportStartFrame);

  fn extract_component((settings, source_handle, start_frame): QueryItem<'_, Self::QueryData>,
  ) -> Option<Self::Out>
  {
    Some((settings.clone(), source_handle.clone_weak(), start_frame.clone()))
  }
}


fn setup_exporters(
    mut commands: Commands,
    exporters: Query<Entity, (With<ImageExportSettings>, Without<ImageExportStartFrame>)>,
    mut frame_id: Local<u64>,
)
{
  *frame_id = frame_id.wrapping_add(1);
  for entity in &exporters
  {
    commands.entity(entity).insert(ImageExportStartFrame(*frame_id));
  }
}


#[derive(Bundle, Default)]
pub struct ImageExportBundle
{
  pub source: Handle<ImageSource>,
  pub settings: ImageExportSettings,
}


fn save_buffer_as_resource(
  export_bundles: Query<(
      &Handle<ImageSource>,
      &ImageExportSettings,
  )>,
  sources: Res<RenderAssets<ImageSource>>,
  render_device: Res<RenderDevice>,
  exported_images: ResMut<ExportedImages>,
  mut frame_id: Local<u64>,
)
{
  *frame_id = frame_id.wrapping_add(1);

  let mut locked_images = exported_images.0.lock();

  if locked_images.is_empty()
  {
    return;
  }

  log::debug!("num of export bundles {}", export_bundles.iter().len());

  let mut futures = Vec::new();

  for (source_handle, _) in &export_bundles
  {
    if let Some(gpu_source) = sources.get(source_handle)
    {
      let slice = gpu_source.buffer.slice(..);

      let (mapping_tx, mapping_rx) = oneshot::channel();

      render_device.map_buffer(&slice, MapMode::Read, move |res|
      {
        mapping_tx.send(res).unwrap();
      });

      futures.push((slice, mapping_rx));
    }
  }

  render_device.poll(Maintain::Wait);
  for ((slice, future), (source_handle, settings)) in futures.iter_mut().zip(export_bundles.iter())
  {
    futures_lite::future::block_on(future).unwrap().unwrap();
    let mut image_bytes = slice.get_mapped_range().to_vec();
    if let Some(gpu_source) = sources.get(source_handle)
    {
      gpu_source.buffer.unmap();
      let (bytes_per_row, padded_bytes_per_row, source_size) = gpu_source.get_bps();

      if bytes_per_row != padded_bytes_per_row
      {
        let mut unpadded_bytes =
            Vec::<u8>::with_capacity(source_size.height as usize * bytes_per_row);

        for padded_row in image_bytes.chunks(padded_bytes_per_row)
        {
          unpadded_bytes.extend_from_slice(&padded_row[..bytes_per_row]);
        }

        image_bytes = unpadded_bytes;
      }

      if let Some(export_img) = locked_images.get_mut(&settings.name)
      {
        let mut buffer = export_img.0.write();
        buffer.update_data(*frame_id, &image_bytes);
      }
    }
  }
}


/// Plugin enabling the generation of image sequences.
#[derive(Default)]
pub struct GpuToCpuCpyPlugin;


#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum ImageExportSystems
{
  SetupImageExport,
  SetupImageExportFlush,
}


impl Plugin for GpuToCpuCpyPlugin
{
  fn build(&self, app: &mut App)
  {
    let exported_images = ExportedImages::default();

    app.insert_resource(exported_images.clone());

    app.configure_sets(
        PostUpdate,
        (SetupImageExport, SetupImageExportFlush).chain().before(CameraUpdateSystem),
    )
    .register_type::<ImageSource>()
    .init_asset::<ImageSource>()
    .register_asset_reflect::<ImageSource>()
    .add_plugins((
      RenderAssetPlugin::<ImageSource>::default(),
      ExtractComponentPlugin::<ImageExportSettings>::default(),
    ))
    .add_systems(
      PostUpdate,
      (
        setup_exporters.in_set(SetupImageExport),
        apply_deferred.in_set(SetupImageExportFlush),
      ),
    );

    let render_app = app.sub_app_mut(RenderApp);

    render_app.insert_resource(exported_images);

    render_app.add_systems(
      Render,
      save_buffer_as_resource.after(RenderSet::Render).before(RenderSet::Cleanup),
    );

    let mut graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();

    graph.add_node(NodeName, ImageExportNode);
    graph.add_node_edge(CameraDriverLabel, NodeName);
  }
}
