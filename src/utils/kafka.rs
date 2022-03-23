use tokio::time::{sleep, Duration};

use rdkafka::{
    //consumer::{BaseConsumer, DefaultConsumerContext},
    client::DefaultClientContext,
    admin::{
        AdminClient, AdminOptions, NewTopic,
        TopicReplication,
    },
    //metadata::Metadata,
    ClientConfig,
    //producer::FutureProducer,
};

use log::*;


pub fn create_config(brokers: &str) -> ClientConfig {
    let mut config = ClientConfig::new();
    config.set("bootstrap.servers", brokers);
    config
}

pub fn create_admin_client(brokers: &str) -> AdminClient<DefaultClientContext> {
    create_config(brokers)
        .create()
        .expect("admin client creation failed")
}

pub async fn reinit_topics(brokers: &str, topics: &[&str]) {
    let opts = AdminOptions::new().operation_timeout(Some(Duration::from_secs(1)));
    let admin_client = create_admin_client(brokers);

    info!("Deleting topics... {:?}", topics);

    admin_client.delete_topics(topics, &opts)
        .await
        .expect("topic deletion failed");

    sleep(Duration::from_secs(3)).await;

    for t in topics.iter() {
        info!("Creating topic: {}", t);
        NewTopic::new(t, 1, TopicReplication::Fixed(1));
        sleep(Duration::from_secs(3)).await;
    }
}
