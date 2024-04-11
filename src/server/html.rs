use {
    crate::{consts::PATH, log::Err, types::Templates},
    std::fs,
};

pub fn cache() -> Templates {
    let meta = get_file(PATH.meta, "");
    Templates {
        nf404: get_file(PATH.nf404, &meta),
        pd403: get_file(PATH.pd403, &meta),
        menu: get_file(PATH.menu, &meta),
        myc_page: get_file(PATH.shroompage, &meta),
    }
}

fn get_file(path: &'static str, meta: &str) -> String {
    match fs::read_to_string(path) {
        Ok(v) => v.replace("{META}", meta),
        Err(e) => {
            format!("{} {}", e, path).log_err();
            "".to_string()
        }
    }
}
