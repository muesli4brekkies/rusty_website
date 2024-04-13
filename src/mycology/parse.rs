use {
    crate::{
        consts,
        log::Err,
        mycology::generate::{CatInfo, GenInfo, Layer, SpecInfo},
        types::{Categories, Condition, Genera, Species, YamlChunks},
    },
    std::fs,
};

trait Construct {
    fn struct_category(self, no_gen: bool) -> Categories;
    fn struct_genus(self) -> Genera;
    fn struct_species(self) -> Species;
}

impl Construct for YamlChunks {
    fn struct_category(self, no_gen: bool) -> Categories {
        self.into_iter()
            .map(|cat| CatInfo {
                label: cat.first().sanitise(),
                title: Some(&cat.iter().fold("".to_string(), |a, b| {
                    match b.trim().starts_with("title:") {
                        true => b.to_string(),
                        false => a,
                    }
                }))
                .sanitise(),
                genera: if no_gen {
                    vec![]
                } else {
                    split_by(cat, condition(Layer::Genus)).struct_genus()
                },
            })
            .collect()
    }

    fn struct_genus(self) -> Genera {
        self.into_iter()
            .map(|gen| GenInfo {
                title: gen.first().sanitise(),
                species: split_by(gen, condition(Layer::Species)).struct_species(),
            })
            .collect()
    }

    fn struct_species(self) -> Species {
        self.into_iter()
            .map(|spe| {
                let mut species = spe.into_iter();
                SpecInfo {
                    title: species.next().as_ref().sanitise(),
                    name: species.next().as_ref().sanitise(),
                    blurb: species.map(|s| (Some(&s)).sanitise()).collect(),
                }
            })
            .collect()
    }
}

trait Sanitise {
    fn sanitise(self) -> String;
}

impl Sanitise for Option<&String> {
    fn sanitise(self) -> String {
        self.unwrap()
            .trim()
            .trim_start_matches("blurb: ")
            .trim_start_matches("common_name: ")
            .trim_start_matches("title: ")
            .replace(':', "")
    }
}

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

fn condition(layer: Layer) -> Condition {
    Box::new(match layer {
        Layer::Category => |(_, s)| !s.starts_with("  ") && s.ends_with(':'),
        Layer::Genus => |(_, s)| s.starts_with("  ") && !s.starts_with("   ") && s.ends_with(':'),
        Layer::Species => |(_, s)| s.starts_with("    ") && s.ends_with(':'),
    })
}

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
