use std::error::Error;
use std::fs::File;
//use std::path::Path;
//
use serde::{Deserialize, Serialize};

#[derive(Debug ,Serialize, Deserialize, PartialEq, Clone)]
pub struct Config {
    pub runner: RunnerConfig,
}

#[derive(Default, Debug ,Serialize, Deserialize, Clone, PartialEq)]
pub struct RunnerConfig {
    #[serde(default)]
    pub scenario_dir: String,
    #[serde(default)]
    pub job_parallel: bool
}

#[allow(dead_code)]
pub fn new(config_file: &str) -> Result<Config, Box<dyn Error>> {
    let f = File::open(config_file)?;
    let config: Config = serde_yaml::from_reader(f)?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_config_file() {
        let expected: Config = Config{
            runner: RunnerConfig{
                scenario_dir: "scenarios".to_string(),
                job_parallel: true
            }
        };

        let config = new(".testrunner.yaml").unwrap();

        assert_eq!(expected, config)
    }
}
