use std::{cmp::min, io::BufWriter, num::NonZeroU32};

use anyhow::Result;
use fast_image_resize as fr;
use image::{
  codecs::{
    avif::AvifEncoder,
    webp::{WebPEncoder, WebPQuality},
  },
  ColorType, ImageEncoder, ImageFormat,
};
use jpegxl_rs::{decoder_builder, image::ToDynamic};
use strum_macros::{AsRefStr, EnumString};

// Blackmagic URSA Mini Pro是一款革命性的数字电影摄影机，搭载 12288 x 6480 12K Super 35传感器
const MAX_WIDTH: u32 = 12288;
const MAX_HEIGHT: u32 = 12288;

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, EnumString, AsRefStr)]
pub enum Ext {
  avif,
  webp,
  jxl,
}

// static INSTANCE: OnceCell<JPEGXL_DECODER> = OnceCell::new();

pub fn resize(
  bin: &[u8],
  mut to_width: u32,
  mut to_height: u32,
  mime: &str,
  ext: &Ext,
) -> Result<Option<Vec<u8>>> {
  let img;

  if let Some(format) = ImageFormat::from_mime_type(mime) {
    img = match image::load_from_memory_with_format(bin, format) {
      Ok(r) => r,
      Err(_) => {
        let format = image::guess_format(bin)?;
        println!("> format {:?}", format);
        image::load_from_memory_with_format(bin, format)?
      }
    };
  } else {
    #[allow(clippy::never_loop)]
    loop {
      if mime == "image/jxl" {
        let decoder = decoder_builder().build()?;
        if let Some(i) = decoder.decode_to_image(bin)? {
          img = i;
          break;
        }
      }
      return Ok(None);
    }
  }

  let width = img.width();
  let height = img.height();

  let mut src_image = fr::Image::from_vec_u8(
    NonZeroU32::new(width).unwrap(),
    NonZeroU32::new(height).unwrap(),
    img.to_rgba8().into_raw(),
    fr::PixelType::U8x4,
  )?;

  if (to_width >= width && (to_height >= height || to_height == 0))
    || (to_height >= height && (to_width >= width || to_width == 0))
  {
    to_width = width;
    to_height = height;
  } else {
    to_width = min(to_width, MAX_WIDTH);
    to_height = min(to_height, MAX_HEIGHT);

    if to_width == 0 {
      if to_height == 0 {
        to_width = width;
        to_height = height;
      } else {
        to_width = ((width * to_height) as f64 / height as f64).round() as u32
      }
    } else if to_height == 0 {
      to_height = ((height * to_width) as f64 / width as f64).round() as u32
    };
  }
  // Multiple RGB channels of source image by alpha channel (not required for the Nearest algorithm)
  let alpha_mul_div = fr::MulDiv::default();
  alpha_mul_div.multiply_alpha_inplace(&mut src_image.view_mut())?;

  // Create container for data of destination image
  let dst_width = NonZeroU32::new(to_width).unwrap();
  let dst_height = NonZeroU32::new(to_height).unwrap();
  let pixel_type = src_image.pixel_type();
  let mut dst_image = fr::Image::new(dst_width, dst_height, pixel_type);

  // Get mutable view of destination image data
  let mut dst_view = dst_image.view_mut();

  // Create Resizer instance and resize source image
  // into buffer of destination image
  let mut resizer = fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3));
  resizer.resize(&src_image.view(), &mut dst_view).unwrap();

  // Divide RGB channels of destination image by alpha
  alpha_mul_div.divide_alpha_inplace(&mut dst_view).unwrap();
  encode_by_ext(dst_image.buffer(), ext, to_width, to_height)
}

fn encode_by_ext(img: &[u8], ext: &Ext, width: u32, height: u32) -> Result<Option<Vec<u8>>> {
  // Write destination image as PNG-file
  let mut result_buf = BufWriter::new(Vec::new());

  /*
  JPEG quality comparison: 80% vs. 90%
  https://sirv.com/help/articles/jpeg-quality-comparison/

  AVIF 和 WebP 编码质量设置
  https://www.industrialempathy.com/posts/avif-webp-quality-settings/
  */

  match ext {
    // https://github.com/kornelski/cavif-rs --speed=n — Encoding speed between 1 (best, but slowest) and 10 (fastest, but a blurry mess), the default value is 4. Speeds 1 and 2 are unbelievably slow, but make files ~3-5% smaller. Speeds 7 and above degrade compression significantly, and are not recommended.
    Ext::avif => AvifEncoder::new_with_speed_quality(&mut result_buf, 5, 64).write_image(
      img,
      width,
      height,
      ColorType::Rgba8,
    )?,
    Ext::webp => WebPEncoder::new_with_quality(&mut result_buf, WebPQuality::lossy(82))
      .write_image(img, width, height, ColorType::Rgba8)?,
    Ext::jxl => {
      todo!();
      // use image::{DynamicImage, ImageBuffer, Rgba};
      // use jpegxl_rs::{
      //   encode::{EncoderResult, EncoderSpeed},
      //   encoder_builder,
      // };
      // let img = DynamicImage::ImageRgba8(
      //   ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width, height, img.into()).unwrap(),
      // );
      //
      // let mut encoder = encoder_builder()
      //   .quality(3.0)
      //   .speed(EncoderSpeed::Kitten)
      //   .build()?;
      // let buffer: EncoderResult<f32> = encoder.encode(&img.to_rgba16(), width, height)?;
      // return Ok(Some(buffer.data));
    }
  };
  Ok(Some(result_buf.into_inner()?))
}
