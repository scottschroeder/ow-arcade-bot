use crate::util::open_json_obj;
use clap::ArgMatches;

use ow_arcade_watcher::{settings::ArcadeBotConfig, watch_and_update, DynamicConfig};
use stupids3::{get, put};

pub fn watcher(_args: &ArgMatches, cfg: &ArcadeBotConfig) -> Result<(), failure::Error> {
    watch_and_update(cfg)
}

pub fn say(args: &ArgMatches, cfg: &ArcadeBotConfig) -> Result<(), failure::Error> {
    let client = discord::create_client(cfg.discord_token()?)?;
    let room = args.value_of("room").unwrap().parse::<u64>()?;
    let msg = args
        .values_of("message")
        .unwrap()
        .collect::<Vec<_>>()
        .join(" ");
    discord::send_message(&client, room, msg)?;
    Ok(())
}

pub fn validate(args: &ArgMatches) -> Result<(), failure::Error> {
    let cfg: DynamicConfig = open_json_obj(args.value_of_os("config").unwrap())?;
    println!("{:#?}", cfg);
    Ok(())
}

pub fn push(args: &ArgMatches, cfg: &ArcadeBotConfig) -> Result<(), failure::Error> {
    let dyncfg: DynamicConfig = open_json_obj(args.value_of_os("config").unwrap())?;
    put(cfg.s3_bucket()?, cfg.s3_key_config()?, &dyncfg)?;
    Ok(())
}

pub fn pull(_args: &ArgMatches, cfg: &ArcadeBotConfig) -> Result<(), failure::Error> {
    let raw_json = get(cfg.s3_bucket()?, cfg.s3_key_config()?)?;
    println!("{}", raw_json);
    Ok(())
}
