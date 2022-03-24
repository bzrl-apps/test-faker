use clap::ArgMatches;

use anyhow::{anyhow, Result};
use log::*;

use crate::config::Config;
use crate::scenario::Scenario;

pub async fn exec_cmd(config: &Config, matches: &ArgMatches<'_>) -> Result<()> {

    let file = match matches.value_of("scenario-file") {
        Some(f) => f,
        None => return Err(anyhow!("You must specify the scenario file (--scenario-file)")),
    };

    let mut scenario = Scenario::new_from_file(&(config.runner.scenario_dir.as_str().to_owned() + "/" + file));

    // scenario == action
    if let Err(e) = scenario.run().await {
        error!("Running scenario: {e}");
        return Err(e);
    }

    Ok(())
}
