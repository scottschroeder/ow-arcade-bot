#[macro_use]
extern crate log;

use chrono::{self, offset::Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;

type GameDiff = (HashSet<u32>, HashSet<u32>);

pub mod arcade_watcher {

    use crate::arcade_state::ArcadeState;
    use crate::owatapi::{fetch_today, GameModesCache};
    use crate::GameMode;
    use serde::{Deserialize, Serialize};
    use std::collections::{HashMap, HashSet};

    pub struct Watcher<T> {
        inner: HashMap<u64, HashSet<u32>>,
        state: T,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WatcherConfig {
        rooms: HashMap<u64, RoomConfig>,
    }

    impl WatcherConfig {
        pub fn walk_rooms(&self) -> impl Iterator<Item = (u64, &Vec<u32>)> + '_ {
            self.rooms.iter().map(|(r, rc)| (*r, &rc.gamemodes))
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RoomConfig {
        gamemodes: Vec<u32>,
    }

    impl<T: ArcadeState> Watcher<T> {
        pub fn new(state: T, watcher_cfg: &WatcherConfig) -> Watcher<T> {
            Watcher {
                inner: watcher_cfg
                    .walk_rooms()
                    .map(|(room, interested)| (room, interested.iter().cloned().collect()))
                    .collect(),
                state,
            }
        }

        pub fn update(&mut self) -> Result<HashMap<u64, Vec<GameMode>>, failure::Error> {
            let arcade = fetch_today()?;
            let current = arcade.tiles.values().map(|gm| gm.id);
            let (added, removed) = self.state.mode_diff(current.clone())?;
            self.state.set_modes(current)?;

            let mut gmcache = GameModesCache::default();
            let mut result = HashMap::new();
            for (room, wadd, _wrem) in self.inner.iter().filter_map(|(room, interested)| {
                let watched_added = added
                    .intersection(&interested)
                    .cloned()
                    .collect::<HashSet<u32>>();
                let watched_removed = removed
                    .intersection(&interested)
                    .cloned()
                    .collect::<HashSet<u32>>();

                if !watched_added.is_empty() {
                    Some((room, watched_added, watched_removed))
                } else {
                    None
                }
            }) {
                let new = gmcache
                    .get()?
                    .iter()
                    .filter(|gm| wadd.contains(&gm.id))
                    .cloned()
                    .collect::<Vec<_>>();
                result.insert(*room, new);
            }

            Ok(result)
        }
    }
}

pub mod arcade_state {
    use super::GameDiff;
    use std::collections::HashSet;
    use stupids3::{get_obj, put, StupidS3Error};

    pub trait ArcadeState {
        fn previous_modes(&self) -> Result<HashSet<u32>, failure::Error>;
        fn set_modes(&mut self, modes: impl Iterator<Item = u32>) -> Result<(), failure::Error>;
        fn mode_diff(&self, modes: impl Iterator<Item = u32>) -> Result<GameDiff, failure::Error> {
            let next = modes.collect::<HashSet<u32>>();
            let prev = self.previous_modes()?;
            Ok((
                next.difference(&prev).cloned().collect(),
                prev.difference(&next).cloned().collect(),
            ))
        }
    }

    pub struct S3State {
        pub bucket: String,
        pub keyname: String,
    }

    impl ArcadeState for S3State {
        fn previous_modes(&self) -> Result<HashSet<u32>, failure::Error> {
            match get_obj::<Vec<u32>, _, _>(&self.bucket, &self.keyname) {
                Ok(r) => Ok(r.into_iter().collect()),
                Err(s3_err) => match s3_err {
                    StupidS3Error::UnknownError { .. } => Err(s3_err.into()),
                    _ => {
                        warn!("got error: {:#?}", s3_err);
                        Ok(HashSet::new())
                    },
                },
            }
        }
        fn set_modes(&mut self, modes: impl Iterator<Item = u32>) -> Result<(), failure::Error> {
            put(&self.bucket, &self.keyname, &modes.collect::<Vec<_>>())?;
            Ok(())
        }
    }
}

pub mod owatapi {
    use super::Arcade;
    use crate::GameMode;
    use reqwest;

    const OWAPI_BASE: &str = "https://overwatcharcade.today/api";
    pub const OWTODAY_URL: &str = "https://overwatcharcade.today/";

    pub fn fetch_today() -> Result<Arcade, failure::Error> {
        Ok(reqwest::get(&format!("{}/today", OWAPI_BASE))?.json()?)
    }

    pub fn fetch_gamemodes() -> Result<Vec<GameMode>, failure::Error> {
        Ok(reqwest::get(&format!("{}/gamemodes", OWAPI_BASE))?.json()?)
    }

    #[derive(Debug, Default)]
    pub struct GameModesCache {
        inner: Option<Vec<GameMode>>,
    }

    impl GameModesCache {
        fn fetch(&mut self) -> Result<&Vec<GameMode>, failure::Error> {
            self.inner = Some(fetch_gamemodes()?);
            Ok(self.inner.as_ref().unwrap())
        }
        pub fn get(&mut self) -> Result<&Vec<GameMode>, failure::Error> {
            if let Some(ref gm) = self.inner {
                Ok(gm)
            } else {
                self.fetch()
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arcade {
    #[serde(with = "owat_api_date")]
    updated_at: chrono::DateTime<Utc>,
    //updated_at: String,
    #[serde(flatten)]
    tiles: HashMap<String, GameMode>,
    by_user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: u32,
    battletag: String,
    avatar: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMode {
    id: u32,
    pub name: String,
    pub players: String,
    code: String,
    pub image: Option<GameImage>,
}

pub fn example_gamemode(wimg: bool) -> GameMode {
    GameMode {
        id: 0,
        name: "Example VS Mode".into(),
        players: "0v0".to_string(),
        code: "examplevsmode".to_string(),
        image: if wimg {
            Some(GameImage {
                normal: "http://overwatcharcade.today/img/modes/6v6competitiveelimination.jpg"
                    .to_string(),
                large: "http://overwatcharcade.today/img/modes_large/6v6competitiveelimination.jpg"
                    .to_string(),
            })
        } else {
            None
        },
    }
}

use std::fmt;

impl fmt::Display for GameMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref img) = self.image {
            write!(f, "{} {} {}", self.players, self.name, img.normal)
        } else {
            write!(f, "{} {}", self.players, self.name)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameImage {
    pub normal: String,
    pub large: String,
}

mod owat_api_date {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod test {
    use crate::Arcade;
    use serde_json;

    const EXAMPLE_TODAY_API_CALL: &str = include_str!("../example_today_api_call.json");

    #[test]
    fn deserialize_today() {
        let _a: Arcade = serde_json::from_str(EXAMPLE_TODAY_API_CALL).unwrap();
    }
}
