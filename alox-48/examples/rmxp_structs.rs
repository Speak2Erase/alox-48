#![allow(dead_code, missing_docs)]

use alox_48::Deserialize;

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
#[marshal(from = "alox_48::Value")]
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
        &alox_48::Symbol::from($string)
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
                        })
                        .collect(),
                })
            }
            Value::Object(obj) if obj.class == "RPG::MoveCommand" => {
                Self::MoveCommand(rpg::MoveCommand {
                    code: obj.fields[symbol!("code")].clone().into_integer().unwrap() as _,
                    parameters: obj.fields[symbol!("parameters")]
                        .clone()
                        .into_array()
                        .unwrap()
                        .into_iter()
                        .map(Into::into)
                        .collect(),
                })
            }
            Value::Float(f) => Self::Float(f as _),
            Value::Array(ary) => Self::Array(
                ary.into_iter()
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
    use alox_48::Deserialize;
    use std::collections::HashMap;

    #[derive(Debug, Deserialize)]
    #[marshal(deny_unknown_fields)]
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

    #[derive(Deserialize, Debug)]
    #[marshal(from = "alox_48::Userdata")]
    pub struct Table3 {
        xsize: usize,
        ysize: usize,
        zsize: usize,
        data: Vec<i16>,
    }

    impl From<alox_48::Userdata> for Table3 {
        fn from(value: alox_48::Userdata) -> Self {
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
        use alox_48::Deserialize;
        mod page {
            use alox_48::Deserialize;

            use super::super::{EventCommand, MoveRoute};

            #[derive(Debug, Deserialize)]
            #[marshal(deny_unknown_fields)]
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

            #[derive(Debug, Deserialize)]
            #[marshal(deny_unknown_fields)]
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
            #[marshal(deny_unknown_fields)]
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
        #[marshal(deny_unknown_fields)]
        pub struct Event {
            pub id: usize,
            pub name: String,
            pub x: i32,
            pub y: i32,
            pub pages: Vec<page::Page>,
        }
    }

    #[derive(Debug, Deserialize)]
    #[marshal(deny_unknown_fields)]
    pub struct MoveRoute {
        pub repeat: bool,
        pub skippable: bool,
        pub list: Vec<MoveCommand>,
    }

    #[derive(Debug, Deserialize)]
    #[marshal(deny_unknown_fields)]
    pub struct AudioFile {
        pub name: String,
        pub volume: u8,
        pub pitch: u8,
    }

    type Parameter = alox_48::Value;

    #[derive(Debug, Deserialize)]
    #[marshal(deny_unknown_fields)]
    pub struct EventCommand {
        pub code: i32,
        pub indent: usize,
        pub parameters: Vec<Parameter>,
    }

    #[derive(Debug, Deserialize)]
    #[marshal(deny_unknown_fields)]
    pub struct MoveCommand {
        pub code: i32,
        pub parameters: Vec<Parameter>,
    }

    #[derive(Default, Debug, Deserialize)]
    pub struct Actor {
        pub id: i32,
        pub name: String,
        pub class_id: i32,
        pub initial_level: i32,
        pub final_level: i32,
        pub exp_basis: i32,
        pub exp_inflation: i32,
        pub character_name: String,
        pub character_hue: i32,
        pub battler_name: String,
        pub battler_hue: i32,
        pub parameters: Table2,
        pub weapon_id: i32,
        pub armor1_id: i32,
        pub armor2_id: i32,
        pub armor3_id: i32,
        pub armor4_id: i32,
        pub weapon_fix: bool,
        pub armor1_fix: bool,
        pub armor2_fix: bool,
        pub armor3_fix: bool,
        pub armor4_fix: bool,
    }

    #[derive(Debug, Default, Deserialize)]
    #[marshal(from = "alox_48::Userdata")]
    pub struct Table2 {
        xsize: usize,
        ysize: usize,
        data: Vec<i16>,
    }

    impl From<alox_48::Userdata> for Table2 {
        fn from(value: alox_48::Userdata) -> Self {
            let u32_slice: &[u32] =
                bytemuck::cast_slice(&value.data[0..std::mem::size_of::<u32>() * 5]);

            assert_eq!(u32_slice[0], 2);
            let xsize = u32_slice[1] as usize;
            let ysize = u32_slice[2] as usize;
            let zsize = u32_slice[3] as usize;
            let len = u32_slice[4] as usize;

            assert_eq!(xsize * ysize * zsize, len);
            let data =
                bytemuck::cast_slice(&value.data[(std::mem::size_of::<u32>() * 5)..]).to_vec();
            assert_eq!(data.len(), len);

            Self { xsize, ysize, data }
        }
    }
}

use std::ops::{Deref, DerefMut};

/// An array that is serialized and deserialized as padded with a None element.
#[derive(Debug, Clone)]
pub struct NilPadded<T>(Vec<T>);

impl<'de, T> alox_48::Deserialize<'de> for NilPadded<T>
where
    T: alox_48::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, alox_48::DeError>
    where
        D: alox_48::DeserializerTrait<'de>,
    {
        struct Visitor<T> {
            _marker: core::marker::PhantomData<T>,
        }

        impl<'de, T> alox_48::de::Visitor<'de> for Visitor<T>
        where
            T: alox_48::Deserialize<'de>,
        {
            type Value = NilPadded<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a nil padded array")
            }

            fn visit_array<A>(self, mut seq: A) -> Result<Self::Value, alox_48::DeError>
            where
                A: alox_48::de::ArrayAccess<'de>,
            {
                let mut values = Vec::with_capacity(seq.len());

                if let Some(v) = seq.next_element::<Option<T>>()? {
                    if v.is_some() {
                        return Err(alox_48::DeError::custom("the first element was not nil"));
                    }
                }

                while let Some(ele) = seq.next_element::<T>()? {
                    values.push(ele);
                }

                Ok(values.into())
            }
        }

        deserializer.deserialize(Visitor {
            _marker: core::marker::PhantomData,
        })
    }
}

impl<T> Deref for NilPadded<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for NilPadded<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Default> Default for NilPadded<T> {
    fn default() -> Self {
        Self(vec![])
    }
}

impl<T> From<Vec<Option<T>>> for NilPadded<T> {
    fn from(value: Vec<Option<T>>) -> Self {
        let mut iter = value.into_iter();

        assert!(
            iter.next()
                .expect("there should be at least one element")
                .is_none(),
            "the array should be padded with nil at the first index"
        );
        Self(iter.flatten().collect())
    }
}

impl<T> From<Vec<T>> for NilPadded<T> {
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}

// appease clippy
fn main() {}
