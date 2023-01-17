#![allow(dead_code, missing_docs)]

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

// Default values
impl Default for Color {
    fn default() -> Self {
        Self {
            red: 255.0,
            green: 255.0,
            blue: 255.0,
            alpha: 255.0,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(from = "alox_48::Value")]
pub enum ParameterType {
    Integer(i32),
    String(String),
    Color(Color),
    Tone(Tone),
    AudioFile(rpg::AudioFile),
    Float(f32),
    MoveRoute(rpg::MoveRoute),
    MoveCommand(rpg::MoveCommand),
    Array(Vec<String>),
    Bool(bool),
}

macro_rules! symbol {
    ($string:literal) => {
        &alox_48::value::Symbol::from($string)
        // &alox_48::Value::Symbol($string.to_string())
    };
}

impl From<alox_48::Value> for ParameterType {
    fn from(value: alox_48::Value) -> Self {
        use alox_48::Value;
        println!("{value:#?}");

        match value {
            Value::Integer(i) => Self::Integer(i as _),
            Value::String(str) => Self::String(str.to_string_lossy().into_owned()),
            Value::Object(obj) if obj.class == "RPG::AudioFile" => {
                Self::AudioFile(rpg::AudioFile {
                    name: obj.fields[symbol!("name")]
                        .clone()
                        .into_string()
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                    volume: obj.fields[symbol!("volume")]
                        .clone()
                        .into_integer()
                        .unwrap() as _,
                    pitch: obj.fields[symbol!("pitch")].clone().into_integer().unwrap() as _,
                })
            }
            Value::Object(obj) if obj.class == "RPG::MoveRoute" => {
                Self::MoveRoute(rpg::MoveRoute {
                    repeat: obj.fields[symbol!("repeat")].clone().into_bool().unwrap(),
                    skippable: obj.fields[symbol!("skippable")]
                        .clone()
                        .into_bool()
                        .unwrap(),
                    list: obj.fields[symbol!("list")]
                        .clone()
                        .into_array()
                        .unwrap()
                        .into_iter()
                        .map(|obj| {
                            let obj = obj.into_object().unwrap();

                            rpg::MoveCommand {
                                code: obj.fields[symbol!("code")].clone().into_integer().unwrap()
                                    as _,
                                parameters: obj.fields[symbol!("parameters")]
                                    .clone()
                                    .into_array()
                                    .unwrap()
                                    .into_iter()
                                    .map(Into::into)
                                    .collect(),
                            }
                            .into()
                        })
                        .collect(),
                })
            }
            Value::Object(obj) if obj.class == "RPG::MoveCommand" => Self::MoveCommand(
                rpg::MoveCommand {
                    code: obj.fields[symbol!("code")].clone().into_integer().unwrap() as _,
                    parameters: obj.fields[symbol!("parameters")]
                        .clone()
                        .into_array()
                        .unwrap()
                        .into_iter()
                        .map(Into::into)
                        .collect(),
                }
                .into(),
            ),
            Value::Float(f) => Self::Float(f as _),
            Value::Array(ary) => Self::Array(
                ary.clone()
                    .into_iter()
                    .map(|v| v.into_string().unwrap().to_string_lossy().into_owned())
                    .collect(),
            ),
            Value::Bool(b) => Self::Bool(b),
            Value::Userdata(data) if data.class == "Color" => {
                let floats = bytemuck::cast_slice(&data.data);

                Self::Color(Color {
                    red: floats[0],
                    green: floats[1],
                    blue: floats[2],
                    alpha: floats[3],
                })
            }
            Value::Userdata(data) if data.class == "Tone" => {
                let floats = bytemuck::cast_slice(&data.data);

                Self::Tone(Tone {
                    red: floats[0],
                    green: floats[1],
                    blue: floats[2],
                    gray: floats[3],
                })
            }
            _ => panic!("Unexpected type {value:#?}"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Tone {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub gray: f32,
}

pub mod rpg {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct Map {
        pub tileset_id: i32,
        pub width: usize,
        pub height: usize,
        pub autoplay_bgm: bool,
        pub bgm: AudioFile,
        pub autoplay_bgs: bool,
        pub bgs: AudioFile,
        pub encounter_list: Vec<i32>,
        pub encounter_step: i32,
        pub data: Table3,
        pub events: HashMap<i32, event::Event>,
    }

    #[derive(Deserialize, Serialize, Debug)]
    #[serde(from = "alox_48::value::Userdata")]
    pub struct Table3 {
        xsize: usize,
        ysize: usize,
        zsize: usize,
        data: Vec<i16>,
    }

    impl From<alox_48::value::Userdata> for Table3 {
        fn from(value: alox_48::value::Userdata) -> Self {
            let u32_slice: &[u32] =
                bytemuck::cast_slice(&value.data[0..std::mem::size_of::<u32>() * 5]);

            assert_eq!(u32_slice[0], 3);
            let xsize = u32_slice[1] as usize;
            let ysize = u32_slice[2] as usize;
            let zsize = u32_slice[3] as usize;
            let len = u32_slice[4] as usize;

            assert_eq!(xsize * ysize * zsize, len);
            let data =
                bytemuck::cast_slice(&value.data[(std::mem::size_of::<u32>() * 5)..]).to_vec();
            assert_eq!(data.len(), len as _);

            Self {
                xsize,
                ysize,
                zsize,
                data,
            }
        }
    }

    pub mod event {
        use serde::Deserialize;
        mod page {
            use serde::{Deserialize, Serialize};

            use super::super::{EventCommand, MoveRoute};

            #[derive(Debug, Deserialize, Serialize)]
            #[serde(deny_unknown_fields)]
            pub struct Condition {
                pub switch1_valid: bool,
                pub switch2_valid: bool,
                pub variable_valid: bool,
                pub self_switch_valid: bool,
                pub switch1_id: usize,
                pub switch2_id: usize,
                pub variable_id: usize,
                pub variable_value: i32,
                pub self_switch_ch: String,
            }

            #[derive(Debug, Deserialize, Serialize)]
            #[serde(deny_unknown_fields)]
            pub struct Graphic {
                pub tile_id: i32,
                pub character_name: String,
                pub character_hue: i32,
                pub direction: i32,
                pub pattern: i32,
                pub opacity: i32,
                pub blend_type: i32,
            }

            #[derive(Debug, Deserialize)]
            #[serde(deny_unknown_fields)]
            pub struct Page {
                pub condition: Condition,
                pub graphic: Graphic,
                pub move_type: usize,
                pub move_speed: usize,
                pub move_frequency: usize,
                pub move_route: MoveRoute,
                pub walk_anime: bool,
                pub step_anime: bool,
                pub direction_fix: bool,
                pub through: bool,
                pub always_on_top: bool,
                pub trigger: i32,
                pub list: Vec<EventCommand>,
            }
        }

        #[derive(Debug, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct Event {
            pub id: usize,
            pub name: String,
            pub x: i32,
            pub y: i32,
            pub pages: Vec<page::Page>,
        }
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct MoveRoute {
        pub repeat: bool,
        pub skippable: bool,
        pub list: Vec<MoveCommand>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct AudioFile {
        pub name: String,
        pub volume: u8,
        pub pitch: u8,
    }

    type Parameter = alox_48::Value;

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct EventCommand {
        pub code: i32,
        pub indent: usize,
        pub parameters: Vec<Parameter>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct MoveCommand {
        pub code: i32,
        pub parameters: Vec<Parameter>,
    }
}

// appease clippy
fn main() {}
