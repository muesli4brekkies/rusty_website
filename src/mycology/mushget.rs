use super::{generate, parse};
use crate::{
    consts,
    types::{CatInfo, Categories, Response, Templates},
};

trait FilterData {
    fn filter_data(&self, data: &str) -> &CatInfo;
}

impl FilterData for Categories {
    fn filter_data(&self, requested_category: &str) -> &CatInfo {
        self.iter()
            .find(|cat| cat.label == requested_category)
            .unwrap()
    }
}

use consts::status;
pub fn gen_shroom_html(requested_category: String, templates: &Templates) -> Response {
    let requested_category = requested_category.replace('/', "");
    let data = parse::yaml(false);
    if requested_category.is_empty() {
        (
            status::HTTP_200,
            "text/html",
            templates
                .menu
                .replace("{MENU}", &generate::menu(&data))
                .as_bytes()
                .to_vec(),
        )
    } else if data
        .iter()
        .map(|cat| &cat.label)
        .any(|label| label == &requested_category)
    {
        (
            status::HTTP_200,
            "text/html",
            templates
                .shroompage
                .replace("{TITLE}", &data.filter_data(&requested_category).title)
                .replace("{DATA}", &data.filter_data(&requested_category).htmlify())
                .as_bytes()
                .to_vec(),
        )
    } else {
        (
            status::HTTP_404,
            "text/html",
            templates.nf404.as_bytes().to_vec(),
        )
    }
}
