use crate::cli::Cli;
use crate::kafka::consumer_groups;
use crate::kafka::metadata;
use crate::state::*;
use anyhow::Result;
use rdkafka::admin::AdminClient;
use rdkafka::client::DefaultClientContext;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::BaseConsumer;
use std::sync::Arc;

pub struct KafkaClient {
    pub admin_client: Arc<AdminClient<DefaultClientContext>>,
    pub consumer: Arc<BaseConsumer>,
}

impl Clone for KafkaClient {
    fn clone(&self) -> Self {
        Self {
            admin_client: self.admin_client.clone(),
            consumer: self.consumer.clone(),
        }
    }
}

impl KafkaClient {
    pub fn new(cli: &Cli) -> Result<Self> {
        let mut admin_config = ClientConfig::new();
        admin_config
            .set("bootstrap.servers", &cli.bootstrap_servers)
            .set("client.id", "kafka-top-admin")
            .set("allow.auto.create.topics", "false")
            .set("metadata.max.age.ms", "30000");
        for kv in &cli.kafka_config {
            if let Some((k, v)) = kv.split_once('=') {
                admin_config.set(k, v);
            }
        }
        let admin_client: AdminClient<DefaultClientContext> = admin_config
            .create()
            .map_err(|e| anyhow::anyhow!("Failed to create Kafka admin client: {}", e))?;

        let mut consumer_config = ClientConfig::new();
        consumer_config
            .set("bootstrap.servers", &cli.bootstrap_servers)
            .set("client.id", "kafka-top-consumer")
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "latest")
            .set("allow.auto.create.topics", "false")
            .set("metadata.max.age.ms", "30000");
        for kv in &cli.kafka_config {
            if let Some((k, v)) = kv.split_once('=') {
                consumer_config.set(k, v);
            }
        }
        let consumer = consumer_config
            .create()
            .map_err(|e| anyhow::anyhow!("Failed to create Kafka consumer: {}", e))?;

        Ok(Self {
            admin_client: Arc::new(admin_client),
            consumer: Arc::new(consumer),
        })
    }

    pub async fn get_dashboard_data(&self) -> Result<DashboardData> {
        let admin = self.admin_client.clone();
        let consumer = self.consumer.clone();
        tokio::task::spawn_blocking(move || {
            let client = KafkaClient { admin_client: admin, consumer };
            metadata::fetch_dashboard_data(&client)
        })
        .await
        .map_err(|e| anyhow::anyhow!("Background task failed: {}", e))?
    }

    pub async fn get_topics(&self, filter: Option<&str>) -> Result<TopicsData> {
        let admin = self.admin_client.clone();
        let consumer = self.consumer.clone();
        let filter = filter.map(|s| s.to_string());
        tokio::task::spawn_blocking(move || {
            let client = KafkaClient { admin_client: admin, consumer };
            metadata::fetch_topics(&client, filter.as_deref())
        })
        .await
        .map_err(|e| anyhow::anyhow!("Background task failed: {}", e))?
    }

    pub async fn get_topic_detail(&self, topic: &str) -> Result<TopicDetailData> {
        let admin = self.admin_client.clone();
        let consumer = self.consumer.clone();
        let topic = topic.to_string();
        tokio::task::spawn_blocking(move || {
            let client = KafkaClient { admin_client: admin, consumer };
            metadata::fetch_topic_detail(&client, &topic)
        })
        .await
        .map_err(|e| anyhow::anyhow!("Background task failed: {}", e))?
    }

    pub async fn list_consumer_groups(&self, filter: Option<&str>) -> Result<GroupsData> {
        let admin = self.admin_client.clone();
        let consumer = self.consumer.clone();
        let filter = filter.map(|s| s.to_string());
        tokio::task::spawn_blocking(move || {
            let client = KafkaClient { admin_client: admin, consumer };
            consumer_groups::fetch_groups(&client, filter.as_deref())
        })
        .await
        .map_err(|e| anyhow::anyhow!("Background task failed: {}", e))?
    }

    pub async fn get_group_detail(&self, group_id: &str) -> Result<GroupDetailData> {
        let admin = self.admin_client.clone();
        let consumer = self.consumer.clone();
        let group_id = group_id.to_string();
        tokio::task::spawn_blocking(move || {
            let client = KafkaClient { admin_client: admin, consumer };
            consumer_groups::fetch_group_detail(&client, &group_id)
        })
        .await
        .map_err(|e| anyhow::anyhow!("Background task failed: {}", e))?
    }
}
