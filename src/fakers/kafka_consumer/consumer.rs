use serde::{Deserialize, Serialize};
//use std::collections::HashMap;
use serde_yaml::{Mapping, Value};
use serde_json::json;

use std::fs::File;
use std::io::Write;

use anyhow::{anyhow, Result, Error};

use futures::Future;
//use futures::FutureExt;
//use futures::pin_mut;
use std::pin::Pin;

use tokio::sync::broadcast::{Sender, Receiver};
//use async_channel::{Sender, Receiver};
//use tokio::runtime::Runtime;
use async_trait::async_trait;

use log::*;

use crate::faker::{Faker, FakerMod};

use rdkafka::{
    config::{ClientConfig, RDKafkaLogLevel},
    consumer::{
        stream_consumer::StreamConsumer,
        Consumer,
        CommitMode,
    },
    message::{Headers, Message},
};

#[derive(Debug ,Serialize, Deserialize, Clone)]
struct Config {
    #[serde(default = "default_group_id")]
    group_id: String,
    #[serde(default = "default_offset")]
    offset: String,
    topics: Vec<String>,
    options: Mapping,
    #[serde(default = "default_loglevel")]
    log_level: String
}

fn default_group_id() -> String {
    "consumer-group-1".to_string()
}

fn default_offset() -> String {
    "earliest".to_string()
}

fn default_loglevel() -> String {
    "info".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Config {
            group_id: "consumer-group-1".to_string(),
            offset: "latest".to_string(),
            topics: vec![],
            options: Mapping::new(),
            log_level: "info".to_string(),
        }
    }
}

// Our plugin implementation
#[derive(Default, Debug ,Serialize, Deserialize, Clone)]
struct KafkaConsumer {
    brokers: Vec<String>,
    config: Config,
    output_file: Option<String>,
}

#[async_trait]
impl FakerMod for KafkaConsumer {
    type Future = Pin<Box<dyn Future<Output = Result<(), Error>> + Send + Sync>>;

    fn validate_params(&self) -> Result<()> {
        if self.brokers.is_empty() {
            return Err(anyhow!("brokers cannnot be empty"));
        }

        if self.config.topics.is_empty() {
            return Err(anyhow!("topics cannot be empty"));
        }

        Ok(())
    }

    fn func(&self, _tx: Sender<bool>, mut _rx: Receiver<bool>) -> Self::Future {
        let _ =  env_logger::try_init();

        let mut client_config = ClientConfig::new();

        client_config.set("group.id", self.config.group_id.clone())
                    .set("bootstrap.servers", self.brokers.join(","))
                    .set("auto.offset.reset", self.config.offset.clone());

        match self.config.log_level.as_str() {
            "debug" => client_config.set_log_level(RDKafkaLogLevel::Debug),
            "info" => client_config.set_log_level(RDKafkaLogLevel::Info),
            "notice" => client_config.set_log_level(RDKafkaLogLevel::Notice),
            "warning" => client_config.set_log_level(RDKafkaLogLevel::Warning),
            "error" => client_config.set_log_level(RDKafkaLogLevel::Error),
            "critical" => client_config.set_log_level(RDKafkaLogLevel::Critical),
            "alert" => client_config.set_log_level(RDKafkaLogLevel::Alert),
            "emerg" => client_config.set_log_level(RDKafkaLogLevel::Emerg),
            _ => client_config.set_log_level(RDKafkaLogLevel::Info),
        };

        for (k, v) in self.config.options.iter() {
            if let Some(s) = v.as_str() {
                client_config.set(k.as_str().unwrap(), s);
            }
        }

        let config = self.config.clone();
        let output_file = self.output_file.clone();

        Box::pin(async move {
            let topics: Vec<&str> = config.topics.iter().map(|t| t.as_ref()).collect();
            let consumer: StreamConsumer  = client_config.create()?;

            consumer.subscribe(&topics)?;

            let mut f_output = match output_file {
                Some(o) => Some(File::create(o)?),
                None => None,
            };

            info!("Start receving messages...");

            // Fuse & pin mut as recommanded in the doc of futures::select!
            //let rx_recv_fut = rx.recv().fuse();
            //let consumer_recv_fut = consumer.recv().fuse();
            //pin_mut!(consumer_recv_fut, rx_recv_fut);

            //loop {
                //futures::select! {
                    //msg = consumer_recv_fut => {
                        //warn!("hmmmmm");
                        //match msg {
                            //Err(_) => { warn!("No message received yet!"); },
                            //Ok(m) => {
                                //debug!("key: 'd{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                                    //m.key(), m.topic(), m.partition(), m.offset(), m.timestamp());

                                //if let Some(headers) = m.headers() {
                                    //for i in 0..headers.count() {
                                        //let header = headers.get(i).unwrap();
                                        //debug!("Header {:#?}: {:?}", header.0, header.1);
                                    //}
                                //}

                                //match m.payload_view::<str>() {
                                    //Some(Ok(payload)) => {
                                        //debug!("Payload received from kafka: {}", payload);

                                        //let value = json!({
                                            //"topic": m.topic(),
                                            //"message": payload,
                                        //});

                                        //if let Some(ref f) = f_output {
                                            //serde_json::to_writer(f, &value)?;
                                        //}

                                        //consumer.commit_message(&m, CommitMode::Async)?;
                                    //},
                                    //Some(Err(e)) => { return Err(anyhow!(e)); },
                                    //None => { return Err(anyhow!("No content received from the topic")); },
                                //};
                            //}
                        //}
                    //}
                    //_ = rx_recv_fut => {
                        //info!("Consumer: got a termination message");
                        ////return Ok(());
                    //},
                    //complete => {
                        //info!("all branches completed");
                        ////return Ok(());
                    //},
                //};
            //}

            loop {
                match consumer.recv().await {
                    Err(e) => { warn!("{e}"); },
                    Ok(m) => {
                        debug!("key: 'd{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                            m.key(), m.topic(), m.partition(), m.offset(), m.timestamp());

                        if let Some(headers) = m.headers() {
                            for i in 0..headers.count() {
                                let header = headers.get(i).unwrap();
                                debug!("Header {:#?}: {:?}", header.0, header.1);
                            }
                        }

                        match m.payload_view::<str>() {
                            Some(Ok(payload)) => {
                                debug!("Payload received from kafka topic {}: {}", m.topic(), payload);

                                if let Some(ref mut f) = f_output {
                                    let value = json!({
                                        "topic": m.topic(),
                                        "message": payload,
                                    });

                                    let text = serde_json::to_string(&value).unwrap();

                                    //if let Err(e) = serde_json::to_writer(f, &value) {
                                        //error!("consumer: {}", e);
                                        //return Err(anyhow!(e));
                                    //}
                                    //serde_json::to_writer(f, b"\n").unwrap();
                                    f.write_all(text.as_bytes()).unwrap();
                                    f.write_all(b"\n").unwrap();
                                }

                                if let Err(e) = consumer.commit_message(&m, CommitMode::Async) {
                                    error!("consumer: {}", e);
                                    return Err(anyhow!(e));
                                }
                            },
                            Some(Err(e)) => {
                                error!("consumer: {}", e);
                                return Err(anyhow!(e));
                            },
                            None => {
                                let e = anyhow!("No content received from the topic");
                                error!("{e}");

                                return Err(e);
                            },
                        };
                    }
                };
            }
        })
    }
}

fn func(params: Mapping, tx: Sender<bool>, rx: Receiver<bool>) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + Sync>> {
    Box::pin(async move {
        let v_params = Value::Mapping(params);

        let consumer: KafkaConsumer = serde_yaml::from_value(v_params)?;

        consumer.validate_params()?;
        consumer.func(tx, rx).await?;

        Ok(())

    })
}

inventory::submit!(Faker {name: "kafka-consumer", func: func });
