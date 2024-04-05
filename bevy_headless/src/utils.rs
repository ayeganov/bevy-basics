use bevy::{
    asset::{Assets, Handle},
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

use crate::{ImageExportBundle, ImageSource, ExportImage, ExportedImages};

#[derive(Default, Resource)]
pub struct CurrImage {
    pub img_buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub frame_id: u64,
    pub extension: String,
}


impl CurrImage {
    pub fn update_data<P, Container>(
        &mut self,
        frame_id: u64,
        image_bytes: &ImageBuffer<P, Container>,
        extension: String,
    ) where
        P: Pixel + image::PixelWithColorType,
        [P::Subpixel]: EncodableLayout,
        Container: Deref<Target = [P::Subpixel]>,
    {
        self.frame_id = frame_id;

        self.extension = extension;

        let (w, h) = image_bytes.dimensions();
        if let Some(rgba_img_buff) = RgbaImage::from_raw(w, h, image_bytes.as_bytes().to_owned()) {
            self.img_buffer = rgba_img_buff;
        } else {
            log::error!("Error updating curr image image buffer");
        };
    }

    pub fn create_path(&self, dir: &str) -> String {
        // shouldn't be in loop, remove later
        std::fs::create_dir_all(dir).expect("Output path could not be created");

        format!("{dir}/{:06}.{}", self.frame_id, self.extension)
    }

    pub fn to_web_base64(&self) -> anyhow::Result<String> {
        base64_browser_img(&self.img_buffer)
    }

    pub fn dimensions(&self) -> [u32; 2] {
        let (w, h) = self.img_buffer.dimensions();
        [w, h]
    }

    pub fn aspect_ratio(&self) -> [u32; 2] {
        let (_w, _h) = self.img_buffer.dimensions();
        // TODO: calculate later
        [16, 9]
    }
}

#[derive(Debug, Default, Resource, Event)]
pub struct SceneInfo {
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


fn next_power_of_2(n: u32) -> usize
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

  let initial_texture_width = cols * view_width;
  let initial_texture_height = rows * view_height;

  let texture_width = next_power_of_2(initial_texture_width);
  let texture_height = next_power_of_2(initial_texture_height);

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
    commands: &mut Commands,
    images: &mut ResMut<Assets<Image>>,
    export_sources: &mut ResMut<Assets<ImageSource>>,
    exported_images: &mut ResMut<ExportedImages>,
    viewport_size: (u32, u32),
    num_views: u32,
) -> (RenderTarget, ExportImage, Vec<(u32, u32)>)
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
          // ?? remove ??
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
  locked_images.push(export_image.clone());

  log::info!("Setup exported images. It has {} images. Address of the container: {:?}", locked_images.len(), locked_images.as_ptr() as *const Vec<ExportImage>);

  commands.spawn(ImageExportBundle {
    source: export_sources.add(render_target_image_handle.clone().into()),
    ..Default::default()
  });

  (RenderTarget::Image(render_target_image_handle), export_image, viewports)
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

fn white_img_placeholder(w: u32, h: u32) -> String {
    let img = RgbaImage::new(w, h);

    // img.iter_mut().for_each(|pixel| *pixel = 255);
    base64_browser_img(&img).unwrap()
}
