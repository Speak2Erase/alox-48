#![allow(dead_code, missing_docs)]

// FIXME: i32 is too big for most values.
// We should use u16 or u8 for most things.
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
        pub data: Table,
        pub events: HashMap<i32, event::Event>,
    }

    #[derive(Deserialize, Serialize, Debug)]
    #[serde(from = "&[u8]")]
    pub struct Table;

    impl<'de> From<&'de [u8]> for Table {
        fn from(_value: &'de [u8]) -> Self {
            Self
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
