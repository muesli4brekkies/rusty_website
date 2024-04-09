use std::{fs, io};

use crate::{consts, types};

use {consts::domains, types::Domain};
pub fn decide(requested_domain: Option<String>) -> Option<Domain> {
    match requested_domain {
        Some(v) => match v.as_str() {
            domains::NO_DOMAIN => Some(Domain::Site),
            domains::MYCOLOGY => Some(Domain::Mycology),
            _ => None,
        },
        None => None,
    }
}

use {
    consts::{status, MIMETYPES},
    types::Response,
};
pub fn get(path: String) -> Result<Response, io::Error> {
    let path = format!("{}{}", consts::PATH.root, &path);
    match fs::metadata(&path) {
        Ok(v) => {
            let path = format!("{}{}", &path, if v.is_dir() { "/index.html" } else { "" });
            match fs::read(&path) {
                Ok(v) => {
                    let file_type = path.split('.').last().unwrap();
                    let mime_type = MIMETYPES.iter().fold("text/plain", |a, (file, mime)| {
                        if file == &file_type {
                            mime
                        } else {
                            a
                        }
                    });

                    Ok((status::HTTP_200, mime_type, v))
                }
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}
