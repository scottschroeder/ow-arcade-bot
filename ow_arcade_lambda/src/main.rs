use std::error::Error;

use lambda_runtime::{error::HandlerError, lambda, Context};
use log::{self};
use serde_derive::{Deserialize, Serialize};

use ow_arcade_watcher;
use ow_arcade_watcher::settings::load;
use ow_arcade_watcher::watch_and_update;

#[derive(Deserialize)]
struct CustomEvent {
    check_previous: bool,
}

#[derive(Serialize)]
struct CustomOutput {
    message: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger(1);
    lambda!(my_handler);
    Ok(())
}

fn my_handler(_e: CustomEvent, _c: Context) -> Result<CustomOutput, HandlerError> {
    let cfg = load()?;
    watch_and_update(&cfg)?;
    Ok(CustomOutput {
        message: "success".to_string(),
    })
}

fn setup_logger(level: u64) {
    let mut builder = pretty_env_logger::formatted_builder();

    let noisy_modules = &[
        "hyper",
        "mio",
        "tokio_core",
        "tokio_reactor",
        "tokio_threadpool",
        "fuse::request",
        "rusoto_core",
        "want",
    ];

    let log_level = match level {
        //0 => log::Level::Error,
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    if level > 1 && level < 4 {
        for module in noisy_modules {
            builder.filter_module(module, log::LevelFilter::Info);
        }
    }

    builder.filter_level(log_level);
    builder.init();
}
