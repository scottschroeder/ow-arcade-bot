#[macro_use]
extern crate log;

use std::hash::Hash;
use chrono::{self, offset::Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;


pub mod arcade_watcher;
pub mod arcade_state;
pub mod owatapi;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodayResponse {
    data: Arcade,
    success: bool,
    message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Arcade {
    is_today: bool,
    created_at: chrono::DateTime<Utc>,
    modes: Vec<GameMode>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMode {
    pub name: String,
    pub players: String,
    pub image: Option<GameImage>,
    pub description: Option<String>,
    pub label: Option<String>,
}

impl PartialEq for GameMode {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.players == other.players
    }
}
impl Eq for GameMode {}
impl Hash for GameMode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(self.name.as_bytes());
        state.write(self.players.as_bytes());
    }
}

// pub fn example_gamemode(wimg: bool) -> GameMode {
//     GameMode {
//         id: 0,
//         name: "Example VS Mode".into(),
//         players: "0v0".to_string(),
//         image: if wimg {
//             Some(GameImage{url: "http://overwatcharcade.today/img/modes/6v6competitiveelimination.jpg".to_string()})
//         } else {
//             None
//         },
//     }
// }

use std::fmt;

impl fmt::Display for GameMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref img) = self.image {
            write!(f, "{} {} {}", self.players, self.name, img.url)
        } else {
            write!(f, "{} {}", self.players, self.name)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GameImage {
    #[serde(flatten)]
    pub url: String,
}

#[cfg(test)]
mod test {
    use crate::TodayResponse;
    use serde_json;

    const EXAMPLE_TODAY_API_CALL: &str = include_str!("../example_today_api_call.json");

    #[test]
    fn deserialize_today() {
        let _a: TodayResponse = serde_json::from_str(EXAMPLE_TODAY_API_CALL).unwrap();
    }
}
