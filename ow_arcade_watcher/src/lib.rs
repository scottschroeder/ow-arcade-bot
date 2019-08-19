use crate::settings::ArcadeBotConfig;

use discord;

use overwatch::arcade_state::S3State;
use overwatch::arcade_watcher::{Watcher, WatcherConfig};
use serde::{Deserialize, Serialize};
use stupids3::get_obj;

pub fn watch_and_update(cfg: &ArcadeBotConfig) -> Result<(), failure::Error> {
    let discord_client = discord::create_client(&cfg.discord_token()?)?;
    let s3_bucket = cfg.s3_bucket()?;
    let bot_cfg: DynamicConfig = get_obj(&s3_bucket, cfg.s3_key_config()?)?;
    let mut watcher = Watcher::new(
        S3State {
            bucket: s3_bucket,
            keyname: cfg.s3_key_gamestate()?,
        },
        &bot_cfg.watcher,
    );
    for (channel, new_gamemodes) in watcher.update()? {
        for gm in new_gamemodes {
            discord::send_gamemode(&discord_client, channel, &gm)?;
        }
    }
    Ok(())
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicConfig {
    pub watcher: WatcherConfig,
}

#[cfg(test)]
mod test {
    use crate::DynamicConfig;
    use serde_json;

    const EXAMPLE_CFG: &str = include_str!("../example_watcher_config.json");

    #[test]
    fn deserialize_today() {
        let _a: DynamicConfig = serde_json::from_str(EXAMPLE_CFG).unwrap();
    }
}

pub mod settings;
