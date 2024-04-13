use {
    crate::{consts, server::run::Templates},
    std::{fs, io},
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

pub trait CheckErr {
    fn check_err(self, templates: &Templates) -> Response;
}

impl CheckErr for Result<Response, io::Error> {
    fn check_err(self, templates: &Templates) -> Response {
        match self {
            Ok(v) => v,
            Err(e) => match e.to_string().contains("Permission denied") {
                true => Response {
                    status: consts::status::HTTP_403,
                    mime_type: "text/html",
                    content: templates.pd403.as_bytes().to_vec(),
                },
                false => Response {
                    status: consts::status::HTTP_404,
                    mime_type: "text/html",
                    content: templates.nf404.as_bytes().to_vec(),
                },
            },
        }
    }
}

pub fn get(path: &String) -> Result<Response, io::Error> {
    let mut full_path = format!("{}{}", consts::PATH.root, &path);
    if fs::metadata(&full_path)?.is_dir() {
        full_path.push_str("/index.html")
    }
    let file_type = full_path.split('.').last().unwrap();
    let mime_type = consts::MIMETYPES
        .into_iter()
        .find(|(file, _)| file == &file_type)
        .map(|(_, mime)| mime)
        .unwrap_or("text/plain");
    Ok(Response {
        status: consts::status::HTTP_200,
        mime_type,
        content: fs::read(full_path)?,
    })
}
