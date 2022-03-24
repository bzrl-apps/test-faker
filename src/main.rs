extern crate lazy_static;
extern crate log;
extern crate clap;

use anyhow::Result;
use log::*;

use clap::{Arg, App};

#[macro_use]
mod utils;
mod config;
mod exec;
mod scenario;
mod fakers;
mod faker;
mod verifiers;
mod verifier;

#[tokio::main]
async fn main() -> Result<()>{
    let matches = App::new("test-faker")
                        .version(env!("CARGO_PKG_VERSION"))
                        .about("Test Faker helps to run fakers & verifiers to perform integration tests!")
                        .setting(clap::AppSettings::TrailingVarArg)
                        .setting(clap::AppSettings::AllowLeadingHyphen)
                        .arg(Arg::with_name("config")
                            .short("c")
                            .long("config")
                            .value_name("FILE")
                            .default_value(".test-faker.yaml")
                            .help("Sets a custom config file")
                            .takes_value(true))
                        .arg(Arg::with_name("scenario-dir")
                            .short("s")
                            .long("--scenario-dir")
                            .takes_value(true)
                            .required(false)
                            .help("Scenario directory"))
                        .subcommand(
                            App::new("exec")
                                .about("Execute a scenario given by a file")
                                .arg(Arg::with_name("scenario-file")
                                    .long("--scenario-file")
                                    .short("f")
                                    .takes_value(true)
                                    .help("Name of the scenario file to execute")))
                        .get_matches();

    env_logger::init();

    // Gets a value for config if supplied by user, or defaults to ".mgr.yaml"
    let config_file = matches.value_of("config");
    let mut config = config::new(config_file.unwrap_or(".test-faker.yaml")).unwrap();
    info!("--- Configuration ---");
    info!("File: {:?}", config_file);
    info!("Content: {:?}", config);

    if let Some(scenario_dir) = matches.value_of("scenario-dir") {
        config.runner.scenario_dir = <&str>::clone(&scenario_dir).to_owned();
    }

    info!("--- Flags ---");
    info!("Scenario directory: {}", config.runner.scenario_dir);

    info!("--- Final configuration ---");
    info!("{:?}", config);

    match matches.subcommand() {
        ("exec", Some(exec_matches)) => return exec::exec_cmd(&config, exec_matches).await,
        _ => panic!("Command not found"),
    }
}
