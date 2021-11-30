use crate::{arcade_state::ArcadeState, owatapi::fetch_today, GameMode};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub struct Watcher<T> {
    inner: HashMap<u64, HashSet<GameMode>>,
    state: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherConfig {
    rooms: HashMap<u64, RoomConfig>,
}

impl WatcherConfig {
    pub fn walk_rooms(&self) -> impl Iterator<Item = (u64, &Vec<GameMode>)> + '_ {
        self.rooms.iter().map(|(r, rc)| (*r, &rc.gamemodes))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomConfig {
    gamemodes: Vec<GameMode>,
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

    pub fn update(&mut self) -> Result<HashMap<u64, HashSet<GameMode>>, failure::Error> {
        let response = fetch_today()?;
        if !response.success {
            failure::bail!("failed to fetch: {:?}", response.message)
        }
        let arcade = response.data;
        let current = arcade.modes.iter();
        let diff = self.state.mode_diff(current.clone())?;
        self.state.set_modes(current)?;

        let mut result = HashMap::new();
        for (room, wadd, _wrem) in self.inner.iter().filter_map(|(room, interested)| {
            let watched_added = diff
                .added
                .intersection(interested)
                .cloned()
                .collect::<HashSet<GameMode>>();
            let watched_removed = diff
                .removed
                .intersection(interested)
                .cloned()
                .collect::<HashSet<GameMode>>();

            if !watched_added.is_empty() {
                Some((room, watched_added, watched_removed))
            } else {
                None
            }
        }) {
            result.insert(*room, wadd);
        }

        Ok(result)
    }
}
