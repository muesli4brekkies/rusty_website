use crate::{consts, log, types};
use std::fs;

use types::CatInfo;
impl CatInfo {
    pub fn htmlify(&self) -> String {
        self.genera.iter().fold("".to_string(), |a, genus| {
            let genus_html = genus
                .species
                .iter()
                .fold("".to_string(), gen_genus(&self.label, &genus.title));
            format!(
                r#"{}
        <details>
            <summary>
                &#x2022;&nbsp;{}
            </summary>{}
        </details>"#,
                a, genus.title, genus_html
            )
        })
    }
}

use {log::Err, types::GenFold};
fn gen_genus<'g>(category: &'g str, genus: &'g str) -> GenFold<'g> {
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
        let n_files =
            match fs::read_dir([consts::IMAGE_DIR, category, genus, &species.title].join("/")) {
                Ok(v) => v.count(),
                Err(e) => {
                    format!(
                        "{} {}{}/{}/{}",
                        e,
                        consts::IMAGE_DIR,
                        category,
                        genus,
                        species.title
                    )
                    .log_err();
                    3
                }
            };
        let species_html =
            (0..n_files).fold("".to_string(), gen_species(category, genus, &species.title));
        format!(
            r#"{}
          <details>
              <summary>
                  {}
              </summary>
              <p class="blurb">
                  {}
              </p>
              <div class="grid">{}
              </div>
          </details>"#,
            a, name, species.blurb, species_html
        )
    })
}

use types::SpecFold;
fn gen_species<'s>(category: &'s str, genus: &'s str, species: &'s str) -> SpecFold<'s> {
    Box::new(move |a, n| {
        let path = format!(
            r#"{}/{}/{}/{}{}{}.jpg"#,
            category, genus, species, genus, species, n
        );
        format!(
            r#"{}
                  <div>
                      <a href="//muon.blog/mycology/Largeimages/{}">
                          <div class="img_box">
                              <img onload="this.style.opacity=1" loading="lazy" src="//muon.blog/mycology/Smallimages/{}">
                          </div>
                      </a>
                  </div>"#,
            a, path, path
        )
    })
}

use types::Categories;
pub fn menu(categories: &Categories) -> String {
    categories.iter().fold("".to_string(), |a, cat| {
        format!(
            r#"{}
          <p class="menu" id="{}">
              <a href="/{}">
                  {}
              </a>
          </p>"#,
            a, cat.label, cat.label, cat.title
        )
    })
}
