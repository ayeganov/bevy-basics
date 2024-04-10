use crate::ImageSource;

use bevy::{
    ecs::world::World,
    render::{
        render_asset::RenderAssets,
        render_graph::{Node, NodeRunError, RenderGraphContext, RenderLabel},
        render_resource::{ImageCopyBuffer, ImageDataLayout},
        renderer::RenderContext,
        texture::Image,
    },
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct NodeName;

pub struct ImageExportNode;
impl Node for ImageExportNode
{
  fn run(
    &self,
    _: &mut RenderGraphContext,
    render_context: &mut RenderContext,
    world: &World,
  ) -> Result<(), NodeRunError>
  {
    for (_, source) in world.resource::<RenderAssets<ImageSource>>().iter()
    {
      if let Some(gpu_image) = world.resource::<RenderAssets<Image>>().get(&source.source_handle)
      {
        render_context.command_encoder().copy_texture_to_buffer(
          gpu_image.texture.as_image_copy(),
          ImageCopyBuffer {
            buffer: &source.buffer,
            layout: ImageDataLayout {
              offset: 0,
              bytes_per_row: Some(source.padded_bytes_per_row),
              rows_per_image: None,
            },
          },
          source.source_size,
        );
      }
    }

    Ok(())
  }
}
