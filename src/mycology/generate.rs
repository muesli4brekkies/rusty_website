use {
    super::parse::{ShroomInfo, Shroompedia},
    crate::{
        consts, html,
        server::response::Response,
        types::{Categories, Content, Result},
    },
    std::io,
};
pub struct CatInfo {
    pub name: String,
    pub menu_label: String,
    pub genera: Vec<GenInfo>,
}

pub struct GenInfo {
    pub name: String,
    pub species: Vec<SpeInfo>,
}

#[derive(Clone)]
pub struct SpeInfo {
    pub name: String,
    pub common_name: String,
    pub blurb: String,
}

trait FillTemplate {
    fn fill_menu(
        &self,
        categories: &Categories,
        shroompedia: &Shroompedia,
        html_frag: &str,
    ) -> Content;
    fn fill_cat(&self, cat: &CatInfo) -> Result<Content>;
    fn fill_shroompage(&self, shroom: &ShroomInfo) -> Result<Content>;
}

impl FillTemplate for String {
    fn fill_menu(
        &self,
        categories: &Categories,
        shroompedia: &Shroompedia,
        html_frag: &str,
    ) -> Content {
        self.replace("{MENU}", &html::menu(categories, html_frag))
            .replace("{SEARCH}", &html::search_opts(shroompedia))
            .replace("{DATA}", &html::search_obj(shroompedia))
            .into_bytes()
    }
    fn fill_cat(&self, cat: &CatInfo) -> Result<Content> {
        Ok(self
            .replace("{TITLE}", &cat.name)
            .replace("{DATA}", &cat.htmlify()?)
            .into_bytes())
    }
    fn fill_shroompage(&self, shroom: &ShroomInfo) -> Result<Content> {
        Ok(self
            .replace(
                "{TITLE}",
                &format!(
                    "{} - {}",
                    &shroom.latin_name,
                    if shroom.info.common_name == "''" {
                        ""
                    } else {
                        &shroom.info.common_name
                    }
                ),
            )
            .replace("{BLURB}", &shroom.info.blurb)
            .replace("{DATA}", &shroom.htmlify()?)
            .into_bytes())
    }
}

pub async fn get(
    (categories, shroompedia): &(Categories, Shroompedia),
    request: &str,
) -> Result<Response> {
    let mime_type = "text/html";

    if request == "/" {
        Ok(Response {
            status: consts::status::HTTP_200,
            mime_type,
            content: html::from_file(consts::PATH.menu)?.fill_menu(
                categories,
                shroompedia,
                &html::from_file(consts::PATH.frag_menu)?,
            ),
        })
    } else if let Some(shroom) = shroompedia.iter().find(|shroom| {
        shroom.url == request
    }) {
        Ok(Response {
            status: consts::status::HTTP_200,
            mime_type,
            content: html::from_file(consts::PATH.shroompage)?.fill_shroompage(shroom)?,
        })
    } else if let Some(cat) = categories
        .iter()
        .find(|cat| {
            cat.name == request.replace("/", "")})
    {
        Ok(Response {
            status: consts::status::HTTP_200,
            mime_type,
            content: html::from_file(consts::PATH.catpage)?.fill_cat(cat)?,
        })
    } else {
        Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            "Not Found",
        )))
    }
}
