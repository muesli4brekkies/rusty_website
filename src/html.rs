use {
    crate::{
        consts::{self, IMAGE_DIR, PATH},
        mycology::{
            generate::{CatInfo, GenInfo, SpeInfo},
            parse::{ShroomInfo, Shroompedia},
        },
        types::{Categories, GenFold, Result, SpecFold},
    },
    std::fs,
};

pub fn menu(categories: &Categories, html_frag: &str) -> String {
    categories.iter().fold(String::new(), |a, cat| {
        format!(
            "{}{}",
            a,
            html_frag
                .replace("{HREF}", &cat.name)
                .replace("{TITLE}", &cat.menu_label)
        )
    })
}

fn flatten_names(shroompedia: &Shroompedia) -> Vec<(&String, &String)> {
    shroompedia
        .iter()
        .flat_map(|shroom| {
            if shroom.info.common_name == "''" {
                vec![(&shroom.latin_name, &shroom.url)]
            } else {
                vec![
                    (&shroom.latin_name, &shroom.url),
                    (&shroom.info.common_name, &shroom.url),
                ]
            }
        })
        .collect()
}

pub fn search_opts(shroompedia: &Shroompedia) -> String {
    format!(
        r#"<div>
      <datalist id="suggestions">
          {}
      </datalist>
      <form method="dialog" id="form" onSubmit="doSearch()">
        <input id="query" placeholder="Search..." aria-label="Search" list="suggestions" autocomplete="off"> 
        <input type="submit" value="Search Shrooms...">
      </form>
    </div>"#,
        shroompedia
            .iter()
            .flat_map(|shroom| if shroom.info.common_name == "''" {
                vec![&shroom.latin_name]
            } else {
                [vec![&shroom.latin_name], vec![&shroom.info.common_name]].concat()
            })
            .fold(String::new(), |a, name| format!(
                "{}<option>{}</option>\n",
                a, name
            ))
    )
}

pub fn search_obj(shroompedia: &Shroompedia) -> String {
    flatten_names(shroompedia)
        .into_iter()
        .fold(String::new(), |a, (spec, url)| {
            format!(r#"{}"{}":"{}","#, a, spec, url,)
        })
}

struct HtmlFrags<'f> {
    category: &'f str,
    genus: &'f str,
    species: &'f str,
}

impl ShroomInfo {
    pub fn htmlify(&self) -> Result<String> {
        let html_frag = &from_file(PATH.frag_species)?;
        let path = format!("{}/{}", IMAGE_DIR, self.url);
        Ok((0..count_dir(path)).fold(String::new(), |a, n| {
            format!(
                "{}{}",
                a,
                html_frag.replace(
                    "{PATH}",
                    &format!(
                        "{}/{}/{}/{}{}{}.jpg",
                        self.cat, self.gen, self.spe, self.gen, self.spe, n
                    )
                )
            )
        }))
    }
}

impl CatInfo {
    pub fn htmlify(&self) -> Result<String> {
        let html_frags = HtmlFrags {
            category: &from_file(PATH.frag_category)?,
            genus: &from_file(PATH.frag_genus)?,
            species: &from_file(PATH.frag_species)?,
        };
        self.genera.iter().try_fold(String::new(), move |a, genus| {
            let genus_html = genus.species.iter().fold(
                String::new(),
                GenInfo::htmlify(&self.name, &genus.name, &html_frags),
            );
            Ok(format!(
                "{}{}",
                a,
                html_frags
                    .category
                    .replace("{TITLE}", &genus.name)
                    .replace("{HTML}", &genus_html)
            ))
        })
    }
}

impl GenInfo {
    fn htmlify<'g>(
        category: &'g str,
        genus: &'g str,
        html_frags: &'g HtmlFrags<'g>,
    ) -> GenFold<'g> {
        Box::new(move |a, species| {
            let name = format!(
                "{}{}",
                species.name,
                if species.common_name == "''" {
                    String::new()
                } else {
                    format!(" - {}", species.common_name)
                }
            );
            let path = [consts::IMAGE_DIR, category, genus, &species.name].join("/");
            let species_html = (0..count_dir(path)).fold(
                String::new(),
                SpeInfo::htmlify(category, genus, &species.name, html_frags),
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
}

impl SpeInfo {
    fn htmlify<'s>(
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
}
fn count_dir(path: String) -> usize {
    match fs::read_dir(&path) {
        Ok(v) => v
            .filter(|f| {
                f.as_ref().is_ok_and(|f| {
                    f.file_name()
                        .into_string()
                        .unwrap_or_default()
                        .ends_with(".jpg")
                })
            })
            .count(),
        Err(e) => {
            eprintln!("{} {}", e, path,);
            0
        }
    }
}
pub fn from_file(path: &'static str) -> Result<String> {
    let meta = &fs::read_to_string(consts::PATH.meta)?;
    Ok(fs::read_to_string(path).map(|v| v.replace("{META}", meta))?)
}
