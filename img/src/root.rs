use std::str::FromStr;

use axum::{
  extract::Path,
  response::{IntoResponse, Response},
};

use crate::{err::Result, img, img::Ext};

const PREFIX: &str = "https://f004.backblazeb2.com/file/xxai-jxl/";
const CONTENT_TYPE: &str = "content-type";

pub fn byte_u32(bin: &[u8]) -> u32 {
  let mut n = 0;
  for b in bin.iter() {
    n *= 10;
    n += u32::from(b - b'0');
  }
  n
}

pub async fn root(Path((args, id)): Path<(String, String)>) -> Result<Response> {
  let hash;
  let ext;

  if let Some(pos) = id.rfind('.') {
    hash = id[..pos].to_owned();
    ext = Ext::from_str(&id[pos + 1..])?;
  } else {
    hash = id;
    ext = Ext::avif;
  }

  let url = PREFIX.to_owned() + &hash;
  let req = reqwest::get(&url).await?;
  let status = req.status();
  let mime = req
    .headers()
    .get(CONTENT_TYPE)
    .unwrap()
    .to_str()?
    .to_owned();

  let args = args.split('-');
  let mut to_width = 0;
  let mut to_height = 0;

  for i in args {
    if i.len() >= 2 {
      let i = i.as_bytes();
      if i[1..].iter().all(|&byte| byte.is_ascii_digit()) {
        let start = i[0];
        let n = byte_u32(&i[1..]);
        if start == b'w' {
          to_width = n;
        } else if start == b'h' {
          to_height = n;
        }
      }
    }
  }

  macro_rules! rt {
    ($content_type:expr, $content:expr) => {
      Ok((status, [(CONTENT_TYPE, $content_type)], $content).into_response())
    };
  }

  let body = req.bytes().await?;

  if status == 200 {
    if let Some(c) = img::resize(&body, to_width, to_height, &mime, &ext)? {
      let content_type = "image/".to_owned() + ext.as_ref();
      return rt!(content_type, c);
    }
  }

  rt!(mime, body)
}
