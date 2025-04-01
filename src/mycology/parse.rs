use {
    crate::{
        consts::{self},
        mycology::generate::{CatInfo, GenInfo, SpeInfo},
        types::{Categories, YamlChunks, YamlLines, YamlString},
    },
    std::fs,
};

enum Layer {
    Category,
    Genus,
    Species,
}

impl Layer {
    fn condition(&self, s: &str) -> bool {
        use Layer::*;
        match self {
            Category => !s.starts_with("  ") && s.ends_with(':'),
            Genus => s.starts_with("  ") && !s.starts_with("   ") && s.ends_with(':'),
            Species => s.starts_with("    ") && s.ends_with(':'),
        }
    }
}

trait Construct {
    fn construct(self) -> (Categories, Shroompedia);
}

impl Construct for YamlString {
    fn construct(self) -> (Categories, Shroompedia) {
        let lines: YamlLines = self.lines().map(str::to_string).collect();
        let (cat_structs, shroompedia) = lines
            .mangle(Layer::Category)
            .iter()
            .map(|cat_chunk| {
                let mut iter = cat_chunk.iter();
                let cat_name = iter.next().sanitise();
                let menu_label = iter.next().sanitise();
                let (gen_structs, shroompedia) = iter
                    .map(String::to_owned)
                    .collect::<YamlLines>()
                    .mangle(Layer::Genus)
                    .iter()
                    .map(|gen_chunk| {
                        let mut iter = gen_chunk.iter();
                        let gen_name = iter.next().sanitise();
                        let (spec_structs, shroompedia): (Vec<SpeInfo>, Shroompedia) = iter
                            .map(String::to_owned)
                            .collect::<YamlLines>()
                            .mangle(Layer::Species)
                            .iter()
                            .map(|spe_chunk| {
                                let mut iter = spe_chunk.iter();
                                let spe_name = iter.next().sanitise();
                                let common_name = iter.next().sanitise();
                                let spe_blurb = iter.next().sanitise();
                                let spe_info = SpeInfo {
                                    name: spe_name.clone(),
                                    common_name: common_name.clone(),
                                    blurb: spe_blurb,
                                };
                                let shroom_info = ShroomInfo {
                                    cat: cat_name.clone(),
                                    gen: gen_name.clone(),
                                    spe: spe_name.clone(),
                                    latin_name: format!("{} {}", gen_name, spe_name),
                                    info: spe_info.clone(),
                                    url: format!("/{}/{}/{}", cat_name, gen_name, spe_name),
                                };
                                (spe_info, shroom_info)
                            })
                            .collect::<(Vec<SpeInfo>, Shroompedia)>();
                        (
                            GenInfo {
                                name: gen_name,
                                species: spec_structs,
                            },
                            shroompedia,
                        )
                    })
                    .collect::<(Vec<GenInfo>, Vec<Shroompedia>)>();
                (
                    CatInfo {
                        name: cat_name,
                        menu_label,
                        genera: gen_structs,
                    },
                    shroompedia.into_iter().flatten().collect(),
                )
            })
            .collect::<(Vec<CatInfo>, Vec<Shroompedia>)>();
        (cat_structs, shroompedia.into_iter().flatten().collect())
    }
}

trait Sanitise {
    fn sanitise(self) -> String;
}

impl Sanitise for Option<&String> {
    fn sanitise(self) -> String {
        self.unwrap_or(&String::new())
            .trim()
            .trim_start_matches("blurb: ")
            .trim_start_matches("title: ")
            .trim_start_matches("common_name: ")
            .trim_start_matches("name: ")
            .replace(':', "")
    }
}

trait Mangle {
    fn mangle(self, layer: Layer) -> YamlChunks;
}

impl Mangle for YamlLines {
    fn mangle(self, layer: Layer) -> YamlChunks {
        let divisions: Vec<usize> = self
            .iter()
            .enumerate()
            .filter_map(|(i, s)| layer.condition(s).then_some(i))
            .collect();
        let m_divs = &divisions;

        divisions
            .iter()
            .enumerate()
            .map(|(i, n)| {
                self[if let Some(v) = m_divs.get(i + 1) {
                    *n..*v
                } else {
                    *n..self.len()
                }]
                .to_vec()
            })
            .collect()
    }
}

pub type Shroompedia = Vec<ShroomInfo>;
pub struct ShroomInfo {
    pub cat: String,
    pub gen: String,
    pub spe: String,
    pub latin_name: String,
    pub info: SpeInfo,
    pub url: String,
}

pub async fn yaml() -> (Categories, Shroompedia) {
    if let Ok(yaml_string) = fs::read_to_string(consts::YAML_FILE) {
        let (cats,shroompediaa) = yaml_string.construct();
        (cats,shroompediaa)
    } else {
        panic!(
            "yaml munching error. :(\n Does the file exist and have correct permissions? - {}",
            consts::YAML_FILE
        );
    }
}
