use {
  crate::{
    consts,
    log::Err,
    mycology::generate::CatInfo,
    server::run::Templates,
    types::{Categories, GenFold, SpecFold},
  },
  std::fs,
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

impl CatInfo {
  pub fn htmlify(&self, templates: &Templates) -> String {
    self.genera.iter().fold("".to_string(), |a, genus| {
      let genus_html = genus.species.iter().fold(
        "".to_string(),
        gen_genus(&self.label, &genus.title, templates),
      );
      format!(
        "{}{}",
        a,
        templates
          .fragments
          .category
          .replace("{TITLE}", &genus.title)
          .replace("{HTML}", &genus_html)
      )
    })
  }
}

fn gen_genus<'g>(category: &'g str, genus: &'g str, templates: &'g Templates) -> GenFold<'g> {
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
      gen_species(category, genus, &species.title, templates),
    );
    format!(
      "{}{}",
      a,
      templates
        .fragments
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
  templates: &'s Templates,
) -> SpecFold<'s> {
  Box::new(move |a, n| {
    let path = format!(
      "{}/{}/{}/{}{}{}.jpg",
      category, genus, species, genus, species, n
    );
    format!(
      "{}{}",
      a,
      templates.fragments.species.replace("{PATH}", &path)
    )
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
