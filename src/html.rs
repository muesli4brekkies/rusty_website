use {
  crate::{
    consts,
    consts::PATH,
    mycology::generate::CatInfo,
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
        .replace("{LABEL}", &cat.label)
        .replace("{TITLE}", &cat.title)
    )
  })
}

struct SearchInfo {
  category: String,
  species: String,
}

pub fn search(categories: &Categories) -> String {
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
    categories
      .iter()
      .flat_map(|category| category
        .genera
        .iter()
        .flat_map(|genera| genera.species.iter().map(|species| SearchInfo {
          category: category.title.clone(),
          species: format!("{} {}", genera.title, species.title)
        })))
      .collect::<Vec<SearchInfo>>()
      .into_iter()
      .fold(String::new(), |a, info| format!(
        "{}\n<option>{}</option>",
        a, info.species,
      ))
  )
}

pub fn data(categories: &Categories) -> String {
  categories
    .iter()
    .flat_map(|category| {
      category.genera.iter().flat_map(|genera| {
        genera.species.iter().map(|species| SearchInfo {
          category: category.label.clone(),
          species: format!("{} {}", genera.title, species.title),
        })
      })
    })
    .collect::<Vec<SearchInfo>>()
    .into_iter()
    .fold(String::new(), |a, info| {
      format!(r#"{}"{}":"{}","#, a, info.species, info.category,)
    })
}

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
    self.genera.iter().try_fold(String::new(), move |a, genus| {
      let genus_html = genus.species.iter().fold(
        String::new(),
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
        String::new()
      } else {
        format!(" - {}", species.name)
      }
    );
    let path = [consts::IMAGE_DIR, category, genus, &species.title].join("/");
    let species_html = (0..count_dir(path)).fold(
      String::new(),
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
      eprintln!("{} {}", e, path,);
      3
    }
  }
}
pub fn from_file(path: &'static str) -> Result<String> {
  let meta = &fs::read_to_string(consts::PATH.meta)?;
  Ok(fs::read_to_string(path).map(|v| v.replace("{META}", meta))?)
}
