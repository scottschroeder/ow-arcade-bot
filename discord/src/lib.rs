use std::fmt::Display;

use overwatch::{owatapi::OWTODAY_URL, GameMode};
use serenity::{builder::CreateMessage, model::id::ChannelId, prelude::*, utils::Colour};

struct Handler;
impl EventHandler for Handler {}

pub fn create_client<S: AsRef<str>>(token: S) -> Result<Client, failure::Error> {
    Ok(Client::new(&token, Handler)?)
}

pub fn send_message<D: Display>(
    client: &Client,
    channel: u64,
    message: D,
) -> Result<(), failure::Error> {
    let chttp = client.cache_and_http.http.clone();
    let ch = ChannelId(channel);
    ch.say(&chttp, message)?;
    Ok(())
}

pub fn send_gamemode(client: &Client, channel: u64, gm: &GameMode) -> Result<(), failure::Error> {
    let chttp = client.cache_and_http.http.clone();
    let ch = ChannelId(channel);
    ch.send_message(&chttp, |m: &mut CreateMessage| {
        m.embed(|e| {
            e.title(&gm.name);
            e.url(OWTODAY_URL);
            e.color(Colour::from_rgb(0x07, 0x85, 0x3e));
            e.field("Players", &gm.players, true);
            if let Some(ref img) = gm.image {
                e.image(&img.url);
            }
            e
        })
    })?;

    Ok(())
}
