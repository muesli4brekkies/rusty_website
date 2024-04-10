use {
  super::{html, parse},
  crate::{
    consts::status,
    types::{CatInfo, Categories, Response, Templates},
  },
};

trait FilterData {
  fn contains(&self, requested_category: &str) -> bool;
  fn filter_data(&self, requested_category: &str) -> &CatInfo;
}

impl FilterData for Categories {
  fn contains(&self, requested_category: &str) -> bool {
    self
      .iter()
      .map(|cat| &cat.label)
      .any(|label| label == requested_category)
  }
  fn filter_data(&self, requested_category: &str) -> &CatInfo {
    self
      .iter()
      .find(|cat| cat.label == requested_category)
      .unwrap()
  }
}

pub fn get(path: &str, templates: &Templates) -> Response {
  let requested_category = path.replace('/', "");
  let mime_type = "text/html";
  let data = parse::yaml(false);
  if requested_category.is_empty() {
    Response {
      status: status::HTTP_200,
      mime_type,
      content: templates
        .menu
        .replace("{MENU}", &html::menu(&data))
        .as_bytes()
        .to_vec(),
    }
  } else if data.contains(&requested_category) {
    let this_data = data.filter_data(&requested_category);
    Response {
      status: status::HTTP_200,
      mime_type,
      content: templates
        .shroompage
        .replace("{TITLE}", &this_data.title)
        .replace("{DATA}", &this_data.htmlify())
        .as_bytes()
        .to_vec(),
    }
  } else {
    Response {
      status: status::HTTP_404,
      mime_type,
      content: templates.nf404.as_bytes().to_vec(),
    }
  }
}
