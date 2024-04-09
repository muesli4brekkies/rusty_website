use crate::{consts, log, types};
use std::fs;

use types::Templates;
pub fn cache() -> Templates {
    use consts::PATH;
    let meta = get_file(PATH.meta, "");
    Templates {
        nf404: get_file(PATH.nf404, &meta),
        pd403: get_file(PATH.pd403, &meta),
        menu: get_file(PATH.menu, &meta),
        shroompage: get_file(PATH.shroompage, &meta),
    }
}

fn get_file(path: &'static str, meta: &str) -> String {
    use log::Err;
    match fs::read_to_string(path) {
        Ok(v) => v.replace("{META}", meta),
        Err(e) => {
            format!("{}", e).log_err();
            "".to_string()
        }
    }
}
