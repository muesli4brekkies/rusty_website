use {
    crate::{
        consts::{status, MIMETYPES, PATH},
        html,
        types::Result,
    },
    std::fs,
};

pub enum Host {
    Site,
    Mycology,
}

pub struct Response {
    pub status: &'static str,
    pub mime_type: &'static str,
    pub content: Vec<u8>,
}

pub fn get(rpath: &String) -> Result<Response> {
    let path = format!("{}{}", PATH.root, &rpath);
    let is_dir = fs::metadata(&path)?.is_dir();
    let wanted_file = format!("{}{}", path, if is_dir { "/index.html" } else { "" });
    let file_type = wanted_file.split('.').last().unwrap_or("");
    let mime_type = MIMETYPES
        .into_iter()
        .fold("text/plain", |a, (b, c)| if b == file_type { c } else { a });
    Ok(Response {
        status: status::HTTP_200,
        mime_type,
        content: fs::read(wanted_file)?,
    })
}

pub trait CheckErr {
    fn replace_err(self) -> Result<Response>;
}

impl CheckErr for Result<Response> {
    fn replace_err(self) -> Result<Response> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => match e.to_string().contains("Permission denied") {
                true => err::pd403(),
                false => err::nf404(),
            },
        }
    }
}

pub mod err {
    use super::*;
    pub fn nf404() -> Result<Response> {
        Ok(Response {
            status: status::HTTP_404,
            mime_type: "text/html",
            content: html::from_file(PATH.nf404)?.into_bytes(),
        })
    }

    pub fn pd403() -> Result<Response> {
        Ok(Response {
            status: status::HTTP_403,
            mime_type: "text/html",
            content: html::from_file(PATH.pd403)?.into_bytes(),
        })
    }
}
