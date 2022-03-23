use log::*;
use tracing::info;
use std::fs::File;

use serde::{Deserialize, Serialize};
use serde_yaml::Mapping;
//use serde_json::Map;
//use serde_json::Value as jsonValue;

use anyhow::Result;

use tokio::sync::*;
use tokio::signal;
use tokio::time::{sleep, Duration};

use crate::faker;
use crate::utils::*;
//use crate::message::Message;

#[derive(Default, Debug ,Serialize, Deserialize, Clone, PartialEq)]
pub struct Scenario {
    #[serde(default)]
    pub name: String,

    #[serde(default)]
    pub setup: Option<Setup>,
    #[serde(default)]
    pub fakers: Vec<Faker>,
    //#[serde(default)]
    //pub verifiers: Vec<Verifier>,
    //
    #[serde(default)]
    pub teardown: Option<Teardown>,

    #[serde(default)]
    pub options: Option<Options>,
}

#[derive(Default, Debug ,Serialize, Deserialize, Clone, PartialEq)]
pub struct Setup {
    #[serde(default)]
    pub kafka_init: Option<KafkaInit>,
}

#[derive(Default, Debug ,Serialize, Deserialize, Clone, PartialEq)]
pub struct KafkaInit {
    #[serde(default)]
    pub brokers: String,
    #[serde(default)]
    pub topics: Vec<String>,
}

#[derive(Default, Debug ,Serialize, Deserialize, Clone, PartialEq)]
pub struct Faker {
    #[serde(default)]
    pub name: String,

    #[serde(default)]
	pub params: Mapping,
}

#[derive(Default, Debug ,Serialize, Deserialize, Clone, PartialEq)]
pub struct Teardown {
}

#[derive(Default, Debug ,Serialize, Deserialize, Clone, PartialEq)]
pub struct Options {
    pub faker_launch_tempo: Option<u64>,
    pub termination_tempo: Option<u64>,
}

impl Scenario {
    pub fn new_from_file(file: &str) -> Self {
        let f = File::open(file).unwrap();
        serde_yaml::from_reader(f).unwrap()
    }

    #[allow(dead_code)]
    pub fn new_from_str(content: &str) -> Self {
        serde_yaml::from_str(content).unwrap()
    }

    pub async fn run(&mut self) -> Result<()> {
        let (tx, mut rx) = broadcast::channel(16);
        let mut faker_launch_tempo = 1;
        let mut termination_tempo = 3;

        if let Some(opts) = &self.options {
            if let Some(t) = opts.faker_launch_tempo {
                faker_launch_tempo = t;
            }

            if let Some(t) = opts.termination_tempo {
                termination_tempo = t;
            }
        }

        info!("Launching setups...");
        if let Some(setup) = &self.setup {
            if let Some(kafka_init) = &setup.kafka_init {
                let topics: Vec<&str> = kafka_init.topics.iter().map(|x| x as &str).collect();
                kafka::reinit_topics(kafka_init.brokers.as_str(), &topics).await;
            }
        }

        info!("Launching fakers...");
        for f in self.fakers.iter() {
            if let Some(f1) = faker::get_faker(f.name.as_str()) {
                info!("Starting faker name: {}, params: {:?}", f.name, f.params);
                let params = f.params.clone();
                let tx_cloned = tx.clone();
                let rx_cloned = tx_cloned.subscribe();
                tokio::spawn(async move {
                    if let Err(e) = (f1.func)(params, tx_cloned, rx_cloned).await {
                        error!("{e}");
                    }
                });
            }

            sleep(Duration::from_secs(faker_launch_tempo)).await;
        }

        tokio::select! {
            _ = signal::ctrl_c() => {
                info!("Ctrl-C received. Stop running.");
                return Ok(());
            }
            _ = rx.recv() => {
                info!("Msg of termination received from fakers. Stop running in {}s.", termination_tempo);
                sleep(Duration::from_secs(termination_tempo)).await;
                return Ok(());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml::Value;

    #[test]
    fn scenario_new_from_str() {
        let content = r#"
name: scenario1

fakers:
  - name: kafka-producer
    params:
      input_file: scenario1_producer.json
  - name: http-server
    params:
      input_file: scenario1_httpserver.json
"#;

        let scenario = Scenario::new_from_str(content);

        let expected = Scenario {
            name: "scenario1".to_string(),
            setup: None,
            fakers: vec![
                Faker {
                    name: "kafka-producer".to_string(),
                    params: yaml_mapping!(
                        "input_file" => Value::String("scenario1_producer.json".to_string())
                    ),
                },
                Faker {
                    name: "http-server".to_string(),
                    params: yaml_mapping!(
                        "input_file" => Value::String("scenario1_httpserver.json".to_string())
                    ),
                },
            ],
            teardown: None,
            options: None,
        };

        assert_eq!(expected, scenario);
    }

    #[tokio::test]
    async fn scenario_run() {
        let _ = env_logger::try_init();

        let content = r#"
name: scenario1

setup:
  kafka_init:
    brokers: localhost:9092
    topics:
    - scenario_topic1
    - scenario_topic2

options:
  faker_launch_tempo: 3
  termination_tempo: 5

fakers:
  - name: kafka-producer
    params:
      brokers:
      - localhost:9092
      options:
        message.timeout.ms: 5000
      messages:
      - topic: scenario_topic1
        key: key1
        message: "hello world 1"
      - topic: scenario_topic2
        key: key2
        message: "hello world 2"
  - name: kafka-consumer
    params:
      brokers:
      - localhost:9092
      config:
        group_id: group1
        topics:
        - scenario_topic1
        - scenario_topic2
        offset: earliest
        options:
          enable.partition.eof: false
          session.timeout.ms: 6000
          enable.auto.commit: false
          auto.commit.interval.ms: 1000
          enable.auto.offset.store: false
          allow.auto.create.topics: true
      output_file: /tmp/consumer.txt
"#;

        let mut scenario = Scenario::new_from_str(content);
        if let Err(e) = scenario.run().await {
            print!("Running scenario: {e}");
        }

        println!("Running scenario: OK");
    }
}
