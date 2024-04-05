use std::sync::Arc;

use crate::node::{ImageExportNode, NODE_NAME};
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
    reflect::{Reflect, TypeUuid},
    render::{
        camera::CameraUpdateSystem,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        main_graph::node::CAMERA_DRIVER,
        render_asset::{PrepareAssetError, RenderAsset, RenderAssetPlugin, RenderAssets},
        render_graph::RenderGraph,
        render_resource::{Buffer, BufferDescriptor, BufferUsages, Extent3d, MapMode},
        renderer::RenderDevice,
        texture::Image, Render, RenderApp, RenderSet
    },
};
use futures::channel::oneshot;
use image::{ImageBuffer, Rgba};

use parking_lot::{Mutex, RwLock};
use wgpu::Maintain;
use ImageExportSystems::{SetupImageExport, SetupImageExportFlush};

#[derive(Asset, Clone, TypeUuid, Default, Reflect)]
#[uuid = "d619b2f8-58cf-42f6-b7da-028c0595f7aa"]
pub struct ImageSource(pub Handle<Image>);

#[derive(Component, Clone, Default, Debug)]
pub struct ExportImage(pub Arc<RwLock<ImageBuffer<Rgba<u8>, Vec<u8>>>>);


impl ExportImage
{
  pub fn new(size: Extent3d) -> Self
  {
    Self(Arc::new(RwLock::new(ImageBuffer::new(size.width, size.height))))
  }
}


#[derive(Clone, Default, Resource)]
pub struct ExportedImages(pub Arc<Mutex<Vec<ExportImage>>>);


impl From<Handle<Image>> for ImageSource {
    fn from(value: Handle<Image>) -> Self {
        Self(value)
    }
}


#[derive(Component, Clone)]
pub struct ImageExportSettings {
    /// The image file extension. "png", "jpeg", "webp", or "exr".
    pub extension: String,
}


#[derive(Clone)]
pub struct GpuImageExport {
    pub buffer: Buffer,
    pub source_handle: Handle<Image>,
    pub source_size: Extent3d,
    pub bytes_per_row: u32,
    pub padded_bytes_per_row: u32,
}

impl GpuImageExport {
    fn get_bps(&self) -> (usize, usize, Extent3d) {
        (self.bytes_per_row as usize, self.padded_bytes_per_row as usize, self.source_size)
    }
}

impl RenderAsset for ImageSource
{
  type ExtractedAsset = Self;
  type Param = (SRes<RenderDevice>, SRes<RenderAssets<Image>>);
  type PreparedAsset = GpuImageExport;

  fn extract_asset(&self) -> Self::ExtractedAsset {
      self.clone()
  }

  fn prepare_asset(
      extracted_asset: Self::ExtractedAsset,
      (device, images): &mut SystemParamItem<Self::Param>,
  ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
      let gpu_image = images.get(&extracted_asset.0).unwrap();

      let size = gpu_image.texture.size();
      let format = &gpu_image.texture_format;
      let bytes_per_row =
          (size.width / format.block_dimensions().0) * format.block_size(None).unwrap();
      let padded_bytes_per_row =
          RenderDevice::align_copy_bytes_per_row(bytes_per_row as usize) as u32;

      let source_size = gpu_image.texture.size();

      Ok(GpuImageExport {
          buffer: device.create_buffer(&BufferDescriptor {
              label: Some("Image Export Buffer"),
              size: (source_size.height * padded_bytes_per_row) as u64,
              usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
              mapped_at_creation: false,
          }),
          source_handle: extracted_asset.0.clone(),
          source_size,
          bytes_per_row,
          padded_bytes_per_row,
      })
  }
}

#[derive(Component, Clone)]
pub struct ImageExportStartFrame(u64);

impl Default for ImageExportSettings
{
  fn default() -> Self
  {
    Self { extension: "png".into() }
  }
}

impl ExtractComponent for ImageExportSettings
{
  type Filter = ();
  type Out = (Self, Handle<ImageSource>, ImageExportStartFrame);
  type Query =
      (&'static Self, &'static Handle<ImageSource>, &'static ImageExportStartFrame);

  fn extract_component((settings, source_handle, start_frame): QueryItem<'_, Self::Query>,
  ) -> Option<Self::Out>
  {
    Some((settings.clone(), source_handle.clone_weak(), start_frame.clone()))
  }
}

fn setup_exporters(
    mut commands: Commands,
    exporters: Query<Entity, (With<ImageExportSettings>, Without<ImageExportStartFrame>)>,
    mut frame_id: Local<u64>,
) {
    *frame_id = frame_id.wrapping_add(1);
    for entity in &exporters {
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
      &ImageExportStartFrame,
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

  log::debug!("num of exported images {}, address of the container {:?}",
             locked_images.len(),
             locked_images.as_ptr() as *const Vec<ExportImage>);

  let mut futures = Vec::new();

  let mut export_img_idx = 0;
  for (source_handle, _start_frame) in &export_bundles
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
  for ((slice, future), (source_handle, _)) in futures.iter_mut().zip(export_bundles.iter())
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

      if let Some(export_img) = locked_images.get_mut(export_img_idx)
      {
        let mut buffer = export_img.0.write();
        buffer.copy_from_slice(&image_bytes);
//        log::info!("Address of buffer: {:?}", export_img.buffer.as_ptr() as *const _);
      }
      else
      {
        return;
      }

      export_img_idx += 1;
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

    graph.add_node(NODE_NAME, ImageExportNode);
    graph.add_node_edge(CAMERA_DRIVER, NODE_NAME);
  }
}
