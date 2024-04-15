use {
  super::html,
  crate::consts::{self, PATH},
  std::{fs, io},
};

pub enum Host {
  Site,
  Mycology,
}

#[derive(Clone)]
pub struct Response {
  pub status: &'static str,
  pub mime_type: &'static str,
  pub content: Vec<u8>,
}

pub fn get(path: &String) -> Result<Response, io::Error> {
  let mut full_path = format!("{}{}", consts::PATH.root, &path);
  if fs::metadata(&full_path)?.is_dir() {
    full_path.push_str("/index.html")
  }
  let file_type = full_path.split('.').last().unwrap();
  let mime_type = consts::MIMETYPES
    .into_iter()
    .fold("text/plain", |a, (b, c)| if b == file_type { c } else { a });
  Ok(Response {
    status: consts::status::HTTP_200,
    mime_type,
    content: fs::read(full_path)?,
  })
}

pub trait CheckErr {
  fn replace_err(self) -> Result<Response, io::Error>;
}

impl CheckErr for Result<Response, io::Error> {
  fn replace_err(self) -> Result<Response, io::Error> {
    match self {
      Ok(v) => Ok(v),
      Err(e) => match e.to_string().contains("Permission denied") {
        true => pd403(),
        false => nf404(),
      },
    }
  }
}

pub fn nf404() -> Result<Response, io::Error> {
  Ok(Response {
    status: consts::status::HTTP_404,
    mime_type: "text/html",
    content: html::from_file(PATH.nf404)?.as_bytes().to_vec(),
  })
}

pub fn pd403() -> Result<Response, io::Error> {
  Ok(Response {
    status: consts::status::HTTP_403,
    mime_type: "text/html",
    content: html::from_file(PATH.pd403)?.as_bytes().to_vec(),
  })
}
