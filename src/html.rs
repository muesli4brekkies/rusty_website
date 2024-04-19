use {
    crate::{
        consts,
        consts::PATH,
        log::Err,
        mycology::generate::CatInfo,
        types::{Categories, GenFold, Result, SpecFold},
    },
    std::fs,
    core::convert::Into,
};

pub fn menu(categories: &Categories, html_frag: &str) -> String {
    categories.iter().fold("".to_string(), |a, cat| {
        format!(
            "{}{}",
            a,
            html_frag
                .replace("{LABEL}", &cat.label)
                .replace("{TITLE}", &cat.title)
        )
    })
}

#[derive(Clone)]
struct HtmlFrags<'f> {
    category: &'f str,
    genus: &'f str,
    species: &'f str,
}

impl CatInfo {
    pub fn htmlify(&self) -> Result<String> {
        let html_frags = HtmlFrags {
            category: &from_file(PATH.frag_category)?,
            genus: &from_file(PATH.frag_genus)?,
            species: &from_file(PATH.frag_species)?,
        };
        self.genera
            .iter()
            .try_fold("".to_string(), move |a, genus| {
                let genus_html = genus.species.iter().fold(
                    "".to_string(),
                    gen_genus(&self.label, &genus.title, &html_frags),
                );
                Ok(format!(
                    "{}{}",
                    a,
                    html_frags
                        .category
                        .replace("{TITLE}", &genus.title)
                        .replace("{HTML}", &genus_html)
                ))
            })
    }
}

fn gen_genus<'g>(category: &'g str, genus: &'g str, html_frags: &'g HtmlFrags<'g>) -> GenFold<'g> {
    Box::new(move |a, species| {
        let name = format!(
            "{}{}",
            species.title,
            if species.name == "''" {
                "".to_string()
            } else {
                format!(" - {}", species.name)
            }
        );
        let path = [consts::IMAGE_DIR, category, genus, &species.title].join("/");
        let species_html = (0..count_dir(path)).fold(
            "".to_string(),
            gen_species(category, genus, &species.title, html_frags),
        );
        format!(
            "{}{}",
            a,
            html_frags
                .genus
                .replace("{NAME}", &name)
                .replace("{BLURB}", &species.blurb)
                .replace("{HTML}", &species_html)
        )
    })
}

fn gen_species<'s>(
    category: &'s str,
    genus: &'s str,
    species: &'s str,
    html_frags: &'s HtmlFrags<'s>,
) -> SpecFold<'s> {
    Box::new(move |a, n| {
        let path = format!(
            "{}/{}/{}/{}{}{}.jpg",
            category, genus, species, genus, species, n
        );
        format!("{}{}", a, html_frags.species.replace("{PATH}", &path))
    })
}

fn count_dir(path: String) -> usize {
    match fs::read_dir(&path) {
        Ok(v) => v.count(),
        Err(e) => {
            format!("{} {}", e, path,).log_err();
            3
        }
    }
}
pub fn from_file(path: &'static str) -> Result<String> {
    let meta = &fs::read_to_string(consts::PATH.meta)?;
    fs::read_to_string(path)
        .map(|v| v.replace("{META}", meta))
        .map_err(Into::into)
}
