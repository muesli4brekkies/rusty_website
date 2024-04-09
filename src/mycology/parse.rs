use crate::{consts, log, types};
use std::fs;

use types::{CatInfo, Categories, Layer, YamlChunks};
trait StructCategory {
    fn struct_category(self, no_gen: bool) -> Categories;
}

impl StructCategory for YamlChunks {
    fn struct_category(self, no_gen: bool) -> Categories {
        self.into_iter()
            .map(|cat| CatInfo {
                label: trim(cat.first()),
                title: trim(Some(&cat.iter().fold("".to_string(), |a, b| {
                    match b.trim().starts_with("title:") {
                        true => b.to_string(),
                        false => a,
                    }
                }))),
                genera: if no_gen {
                    vec![]
                } else {
                    split_by(cat, condition(Layer::Genus)).struct_genus()
                },
            })
            .collect()
    }
}

use types::{GenInfo, Genera};
trait StructGenus {
    fn struct_genus(self) -> Genera;
}

impl StructGenus for YamlChunks {
    fn struct_genus(self) -> Genera {
        self.into_iter()
            .map(|gen| GenInfo {
                title: trim(gen.first()),
                species: split_by(gen, condition(Layer::Species)).struct_species(),
            })
            .collect()
    }
}

use types::{SpecInfo, Species};
trait StructSpecies {
    fn struct_species(self) -> Species;
}

impl StructSpecies for YamlChunks {
    fn struct_species(self) -> Species {
        self.into_iter()
            .map(|spe| {
                let mut species = spe.into_iter();
                SpecInfo {
                    title: trim(species.next().as_ref()),
                    name: trim(species.next().as_ref()),
                    blurb: species.map(|s| trim(Some(&s))).collect(),
                }
            })
            .collect()
    }
}

use log::Err;
pub fn yaml(get_just_cats: bool) -> Categories {
    match fs::read_to_string(consts::YAML_FILE) {
        Ok(v) => {
            let yaml = v.split('\n').map(str::to_string).collect();
            let categories = split_by(yaml, condition(Layer::Category));
            categories.struct_category(get_just_cats)
        }
        Err(e) => {
            format!("yaml munching error. :( - {} {}", e, consts::YAML_FILE).log_err();
            vec![]
        }
    }
}

use types::Condition;
fn split_by(lines: Vec<String>, condition: Condition) -> YamlChunks {
    let divisions: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(condition)
        .map(|(i, _)| i)
        .collect();
    let m_divs = &divisions;

    divisions
        .iter()
        .enumerate()
        .map(|(i, n)| {
            lines[match m_divs.get(i + 1) {
                Some(v) => *n..*v,
                None => *n..lines.len(),
            }]
            .to_vec()
        })
        .collect()
}

fn condition(layer: Layer) -> Condition {
    Box::new(match layer {
        Layer::Category => |(_, s)| !s.starts_with("  ") && s.ends_with(':'),
        Layer::Genus => |(_, s)| s.starts_with("  ") && !s.starts_with("   ") && s.ends_with(':'),
        Layer::Species => |(_, s)| s.starts_with("    ") && s.ends_with(':'),
    })
}

fn trim(str: Option<&String>) -> String {
    str.unwrap()
        .trim()
        .trim_start_matches("blurb: ")
        .trim_start_matches("common_name: ")
        .trim_start_matches("title: ")
        .replace(':', "")
}
