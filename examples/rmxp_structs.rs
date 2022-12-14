#![allow(dead_code, missing_docs)]

pub mod rpg {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, Deserialize)]
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
        pub struct Event {
            pub id: usize,
            pub name: String,
            pub x: i32,
            pub y: i32,
            pub pages: Vec<page::Page>,
        }
    }

    #[derive(Debug, Deserialize)]
    pub struct MoveRoute {
        pub repeat: bool,
        pub skippable: bool,
        pub list: Vec<MoveCommand>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct AudioFile {
        pub name: String,
        pub volume: u8,
        pub pitch: u8,
    }

    #[derive(Debug, Deserialize)]
    #[allow(missing_docs)]
    pub struct EventCommand {
        pub code: i32,
        pub indent: usize,
        pub parameters: Vec<alox_48::Value>,
    }

    #[derive(Debug, Deserialize)]
    #[allow(missing_docs)]
    pub struct MoveCommand {
        pub code: i32,
        pub parameters: Vec<alox_48::Value>,
    }
}

// appease clippy
fn main() {}
