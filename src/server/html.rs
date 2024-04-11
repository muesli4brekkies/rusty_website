use {
  crate::{
    consts::PATH,
    log::Err,
    server::run::{Fragments, Templates},
  },
  std::fs,
};

pub fn cache() -> Templates {
  let meta = get_file(PATH.meta, "");
  Templates {
    nf404: get_file(PATH.nf404, &meta),
    pd403: get_file(PATH.pd403, &meta),
    menu: get_file(PATH.menu, &meta),
    myc_page: get_file(PATH.shroompage, &meta),
    fragments: Fragments {
      category: get_file(PATH.frag_category, &meta),
      genus: get_file(PATH.frag_genus, &meta),
      species: get_file(PATH.frag_species, &meta),
      menu: get_file(PATH.frag_menu, &meta),
    },
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
