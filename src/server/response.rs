use {
  crate::{
    consts,
    types::{Response, Templates},
  },
  std::{fs, io},
};

pub fn get(path: &String) -> Result<Response, io::Error> {
  let full_path = format!("{}{}", consts::PATH.root, &path);
  let indexed_path = format!(
    "{}{}",
    &full_path,
    if fs::metadata(&full_path)?.is_dir() {
      "/index.html"
    } else {
      ""
    }
  );
  let file_type = indexed_path.split('.').last().unwrap();
  let mime_type = consts::MIMETYPES
    .iter()
    .fold("text/plain", |a, (file, mime)| match file == &file_type {
      true => mime,
      false => a,
    });
  Ok(Response {
    status: consts::status::HTTP_200,
    mime_type,
    content: fs::read(indexed_path)?,
  })
}

pub trait CheckErr {
  fn check_err(self, templates: &Templates) -> Response;
}

impl CheckErr for Result<Response, io::Error> {
  fn check_err(self, templates: &Templates) -> Response {
    match self {
      Ok(v) => v,
      Err(e) => {
        if e.to_string().contains("Permission denied") {
          Response {
            status: consts::status::HTTP_403,
            mime_type: "text/html",
            content: templates.pd403.as_bytes().to_vec(),
          }
        } else {
          Response {
            status: consts::status::HTTP_404,
            mime_type: "text/html",
            content: templates.nf404.as_bytes().to_vec(),
          }
        }
      }
    }
  }
}
