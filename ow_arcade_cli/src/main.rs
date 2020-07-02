#[macro_use]
extern crate log;

use clap::{App, Arg};
use failure;
use failure::bail;

use ow_arcade_watcher::settings::load;
use pretty_env_logger;

const VERSION: &str = env!("CARGO_PKG_VERSION");

mod subcommand;

fn run() -> Result<(), failure::Error> {
    let args = get_args();
    setup_logger(args.occurrences_of("verbosity"));

    let cfg = load()?;

    trace!("Args: {:#?}", &args);
    trace!("Cfg: {:#?}", &cfg);

    match args.subcommand() {
        ("watcher", Some(sub_m)) => subcommand::watcher(sub_m, &cfg)?,
        ("say", Some(sub_m)) => subcommand::say(sub_m, &cfg)?,
        ("config", Some(sub_m)) => match sub_m.subcommand() {
            ("validate", Some(sub_m)) => subcommand::validate(sub_m)?,
            ("pull", Some(sub_m)) => subcommand::pull(sub_m, &cfg)?,
            ("push", Some(sub_m)) => subcommand::push(sub_m, &cfg)?,
            ("", _) => bail!("Please provide a command:\n{}", args.usage()),
            subc => bail!("Unknown command: config: {:?}\n{}", subc, args.usage()),
        },
        ("", _) => bail!("Please provide a command:\n{}", args.usage()),
        subc => bail!("Unknown command: {:?}\n{}", subc, args.usage()),
    }

    Ok(())
}

mod util {
    use serde::de::DeserializeOwned;
    use std::fs;
    use std::path::Path;

    pub fn open_json_obj<D: DeserializeOwned, P: AsRef<Path>>(
        path: P,
    ) -> Result<D, failure::Error> {
        let f = fs::File::open(path.as_ref())?;
        Ok(serde_json::from_reader(f)?)
    }
}

fn main() {
    match run() {
        Ok(_) => (),
        Err(e) => eprintln!("{}", pretty_error(&e)),
    }
}

/// Return a prettily formatted error, including its entire causal chain.
fn pretty_error(err: &failure::Error) -> String {
    let mut pretty = err.to_string();
    let mut prev = err.as_fail();
    while let Some(next) = prev.cause() {
        pretty.push_str(": ");
        pretty.push_str(&next.to_string());
        prev = next;
    }
    pretty
}

fn setup_logger(level: u64) {
    let mut builder = pretty_env_logger::formatted_builder();

    let noisy_modules = &[
        "hyper",
        "mio",
        "tokio_core",
        "tokio_io",
        "tokio_reactor",
        "tokio_threadpool",
        "fuse::request",
        "rusoto_core",
        "rustls",
        "h2",
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

fn get_args() -> clap::ArgMatches<'static> {
    App::new("Overwatch Arcade Bot")
        .about("A Discord chatbot for the Overwatch Arcade")
        .version(VERSION)
        .setting(clap::AppSettings::DeriveDisplayOrder)
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .global(true)
                .help("Sets the level of verbosity"),
        )
        .subcommand(
            clap::SubCommand::with_name("watcher")
                .about("Check and update arcade status")
                .arg(
                    Arg::with_name("config")
                        .short("c")
                        .long("config")
                        .takes_value(true)
                        .help("Provide a file with the program's configuration"),
                ),
        )
        .subcommand(
            clap::SubCommand::with_name("say")
                .about("Have the bot say something in chat")
                .arg(
                    Arg::with_name("room")
                        .long("room")
                        .takes_value(true)
                        .required(true)
                        .help("discord room ID"),
                )
                .arg(
                    Arg::with_name("message")
                        .required(true)
                        .index(1)
                        .multiple(true)
                        .takes_value(true)
                        .help("chat message"),
                ),
        )
        .subcommand(
            clap::SubCommand::with_name("config")
                .about("Manage the configuration object")
                .subcommand(
                    clap::SubCommand::with_name("validate")
                        .about("Validate the config object")
                        .arg(
                            Arg::with_name("config")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help("Provide a file with the program's configuration"),
                        ),
                )
                .subcommand(clap::SubCommand::with_name("pull").about("Grab the current config"))
                .subcommand(
                    clap::SubCommand::with_name("push")
                        .about("Push a new config")
                        .arg(
                            Arg::with_name("config")
                                .required(true)
                                .index(1)
                                .takes_value(true)
                                .help("Provide a file with the program's configuration"),
                        ),
                ),
        )
        .get_matches()
}
