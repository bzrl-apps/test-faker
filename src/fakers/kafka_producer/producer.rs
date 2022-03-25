use serde::{Deserialize, Serialize};
use serde_yaml::{Value, Mapping};

use tokio::sync::broadcast::{Sender, Receiver};
//use async_channel::{Sender, Receiver};
//use tokio::runtime::Runtime;
use async_trait::async_trait;

use std::time::Duration;

use anyhow::{anyhow, Result, Error};

use log::*;

use rdkafka::{
    config::{ClientConfig, RDKafkaLogLevel},
    producer::future_producer::{FutureProducer, FutureRecord},
};

use futures::Future;
use std::pin::Pin;

use crate::faker::{Faker, FakerMod};

// Our plugin implementation
#[derive(Default, Debug ,Serialize, Deserialize, Clone)]
struct KafkaProducer {
    brokers: Vec<String>,
    options: Mapping,
    messages: Vec<Message>,
    #[serde(default = "default_loglevel")]
    log_level: String,
}

fn default_loglevel() -> String {
    "info".to_string()
}

#[derive(Default, Debug ,Serialize, Deserialize, Clone)]
struct Message {
    topic: String,
    key: String,
    message: String,
}

#[async_trait]
impl FakerMod for KafkaProducer {
    type Future = Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>;

    fn validate_params(&self) -> Result<()> {
        if self.brokers.is_empty() {
            return Err(anyhow!("brokers cannot be empty"));
        }

        if self.messages.is_empty() {
            return Err(anyhow!("messages cannot be empty"));
        }

        for m in self.messages.iter() {
            if m.topic.is_empty() || m.message.is_empty() {
                return Err(anyhow!("Topic or message must not be empty!"));
            }
        }

        Ok(())
    }

    fn func(&self, tx: Sender<bool>, _rx: Receiver<bool>) -> Self::Future {
       let _ =  env_logger::try_init();

        let mut client_config = ClientConfig::new();

        client_config.set("bootstrap.servers", self.brokers.join(","));

        match self.log_level.as_str() {
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

        for (k, v) in self.options.iter() {
            if let Some(s) = v.as_str() {
                client_config.set(k.as_str().unwrap(), s);
            }
        }

        let messages = self.messages.clone();

        Box::pin(async move {

            let producer: FutureProducer  = client_config.create()?;

            for msg in messages.clone().iter() {
                info!("Sending messages to the topic {:?}", msg.topic);

                let mut fr = FutureRecord::to(msg.topic.as_str())
                        .payload(msg.message.as_bytes());

                if !msg.key.is_empty() {
                    fr =  fr.key(msg.key.as_bytes());
                }

                let produce_future = producer.send(
                    fr,
                    //.headers(OwnedHeaders::new().add("header_key", "header_value")),
                    Duration::from_secs(0),
                );

                match produce_future.await {
                    Ok(delivery) => {
                        debug!("Kafka producer sent delivery status: {:?}", delivery);
                    }
                    Err((e, _)) => {
                        error!("Kafka producer sent error: {:?}", e);
                        return Err(anyhow!(e));
                    }
                };
            }

            tx.send(true)?;

            Ok(())
        })
    }
}

fn func(params: Mapping, tx: Sender<bool>, rx: Receiver<bool>) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>> {
    Box::pin(async move {
        let v_params = Value::Mapping(params);

        let producer: KafkaProducer = serde_yaml::from_value(v_params)?;

        producer.validate_params()?;
        producer.func(tx, rx).await?;

        Ok(())

    })
}

inventory::submit!(Faker {name: "kafka-producer", func: func });
