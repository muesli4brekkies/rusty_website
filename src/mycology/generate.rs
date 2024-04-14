use {
  super::{html, parse},
  crate::{
    consts::status,
    server::{response::Response, run::Templates},
    types::{Categories, Content},
  },
};

pub struct CatInfo {
  pub title: String,
  pub label: String,
  pub genera: Vec<GenInfo>,
}

pub struct GenInfo {
  pub title: String,
  pub species: Vec<SpecInfo>,
}

pub struct SpecInfo {
  pub title: String,
  pub name: String,
  pub blurb: String,
}

trait FilterData {
  fn contains(&self, requested_category: &str) -> bool;
  fn filter_data(self, requested_category: &str) -> CatInfo;
}

impl FilterData for Categories {
  fn contains(&self, requested_category: &str) -> bool {
    self
      .iter()
      .map(|cat| &cat.label)
      .any(|label| label == requested_category)
  }
  fn filter_data(self, requested_category: &str) -> CatInfo {
    self
      .into_iter()
      .find(|cat| cat.label == requested_category)
      .unwrap()
  }
}

trait FillTemplate {
  fn fill_menu(&self, categories: Categories, html_frag: &str) -> Content;
  fn fill_myc(&self, data: CatInfo, templates: &Templates) -> Content;
}

impl FillTemplate for String {
  fn fill_menu(&self, categories: Categories, html_frag: &str) -> Content {
    self
      .replace("{MENU}", &html::menu(&categories, html_frag))
      .as_bytes()
      .to_vec()
  }
  fn fill_myc(&self, data: CatInfo, templates: &Templates) -> Content {
    self
      .replace("{TITLE}", &data.title)
      .replace("{DATA}", &data.htmlify(templates))
      .as_bytes()
      .to_vec()
  }
}

pub fn get(path: &str, templates: &Templates) -> Response {
  let requested_category = path.replace('/', "");
  let mime_type = "text/html";
  let categories = parse::yaml(true);
  if requested_category.is_empty() {
    Response {
      status: status::HTTP_200,
      mime_type,
      content: templates
        .menu
        .fill_menu(categories, &templates.fragments.menu),
    }
  } else if categories.contains(&requested_category) {
    let data = parse::yaml(false).filter_data(&requested_category);
    Response {
      status: status::HTTP_200,
      mime_type,
      content: templates.myc_page.fill_myc(data, templates),
    }
  } else {
    Response {
      status: status::HTTP_404,
      mime_type,
      content: templates.nf404.as_bytes().to_vec(),
    }
  }
}
