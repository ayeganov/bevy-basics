use bevy::{
    asset::Assets,
    ecs::{
        event::Event,
        system::{Commands, ResMut, Resource},
    },
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        texture::Image,
    },
};
use std::{io::Cursor, ops::Deref};

use base64::{engine::general_purpose, Engine};
use image::{EncodableLayout, ImageBuffer, ImageOutputFormat, Pixel, Rgba, RgbaImage};

use crate::{ImageExportBundle, ImageSource, ExportImage, ExportedImages, ImageExportSettings};


#[derive(Clone, Default, Debug)]
pub struct ImageWrapper
{
  pub img_buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
  pub frame_id: u64,
}


impl ImageWrapper
{
  pub fn new(size: Extent3d) -> Self
  {
    Self
    {
      img_buffer: ImageBuffer::new(size.width, size.height),
      frame_id: 0,
    }
  }
}


impl ImageWrapper
{
  pub fn update_data(
    &mut self,
    frame_id: u64,
    image_bytes: &Vec<u8>,
  )
  {
    self.frame_id = frame_id;
    self.img_buffer.copy_from_slice(image_bytes);
  }
}


#[derive(Debug, Default, Resource, Event)]
pub struct SceneInfo
{
  width: u32,
  height: u32,
}

impl SceneInfo {
    pub fn new(width: u32, height: u32) -> SceneInfo {
        SceneInfo { width, height }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}


fn next_power_of_2(n: usize) -> usize
{
  if n == 0
  {
    1
  }
  else
  {
    2_usize.pow((n - 1).next_power_of_two().trailing_zeros())
  }
}


fn calculate_grid_dimensions(view_width: u32,
                             view_height: u32,
                             num_views: u32)
  -> ((usize, usize), Vec<(u32, u32)>)
{
  let cols = (num_views as f64).sqrt().ceil() as u32;
  let mut rows = (num_views as f64 / cols as f64).ceil() as u32;

  while cols * (rows - 1) >= num_views
  {
      rows -= 1;
  }

  let initial_texture_width = (cols * view_width) as usize;
  let initial_texture_height = (rows * view_height) as usize;

  let texture_width = {
    let is_already_power_of_2 = initial_texture_width & (initial_texture_width - 1) == 0;
    if is_already_power_of_2
    {
      initial_texture_width
    }
    else
    {
      next_power_of_2(initial_texture_width)
    }
  };

  let texture_height = {
    let is_already_power_of_2 = initial_texture_height & (initial_texture_height - 1) == 0;
    if is_already_power_of_2
    {
      initial_texture_height
    }
    else
    {
      next_power_of_2(initial_texture_height)
    }
  };

  let mut positions: Vec<(u32, u32)> = Vec::with_capacity(num_views as usize);
  for i in 0..num_views
  {
    let row = i / cols;
    let col = i % cols;
    let x = col * view_width;
    let y = row * view_height;
    positions.push((x, y));
  }

  ((texture_width, texture_height), positions)
}


pub fn setup_render_target(
    target_name: &String,
    commands: &mut Commands,
    images: &mut ResMut<Assets<Image>>,
    export_sources: &mut ResMut<Assets<ImageSource>>,
    exported_images: &mut ResMut<ExportedImages>,
    viewport_size: (u32, u32),
    num_views: u32,
) -> (RenderTarget, Vec<(u32, u32)>)
{
  let ((tex_width, tex_height), viewports) = calculate_grid_dimensions(viewport_size.0, viewport_size.1, num_views);
  let size = Extent3d
  {
    width: tex_width as u32,
    height: tex_height as u32,
    ..Default::default()
  };

  log::info!("Texture size: {:?}, viewport size: {:?}, num views: {}", size, viewport_size, num_views);

  let mut render_target_image = Image
  {
    texture_descriptor: TextureDescriptor
    {
      label: None,
      size,
      dimension: TextureDimension::D2,
      format: TextureFormat::Rgba8UnormSrgb,
      mip_level_count: 1,
      sample_count: 1,
      usage: TextureUsages::COPY_SRC
          | TextureUsages::COPY_DST
          | TextureUsages::TEXTURE_BINDING
          | TextureUsages::RENDER_ATTACHMENT,
      view_formats: &[],
    },
    ..Default::default()
  };
  render_target_image.resize(size);
  let render_target_image_handle = images.add(render_target_image);

  let export_image = ExportImage::new(size);
  let mut locked_images = exported_images.0.lock();
  locked_images.insert(target_name.clone(), export_image.clone());

//  log::info!("Setup exported images. It has {} images. Address of the container: {:?}", locked_images.len(), locked_images.as_ptr() as *const Vec<ExportImage>);

  commands.spawn(ImageExportBundle {
    source: export_sources.add(render_target_image_handle.clone()),
    settings: ImageExportSettings::new(target_name.clone()),
    ..Default::default()
  });

  (RenderTarget::Image(render_target_image_handle), viewports)
}


fn base64_browser_img<P, Container>(img: &ImageBuffer<P, Container>) -> anyhow::Result<String>
where
  P: Pixel + image::PixelWithColorType,
  [P::Subpixel]: EncodableLayout,
  Container: Deref<Target = [P::Subpixel]>,
{
  let mut image_data: Vec<u8> = Vec::new();
  img.write_to(&mut Cursor::new(&mut image_data), ImageOutputFormat::WebP)?;
  let res_base64 = general_purpose::STANDARD.encode(image_data);
  Ok(format!("data:image/webp;base64,{}", res_base64))
}


fn white_img_placeholder(w: u32, h: u32) -> String
{
  let img = RgbaImage::new(w, h);

  // img.iter_mut().for_each(|pixel| *pixel = 255);
  base64_browser_img(&img).unwrap()
}
