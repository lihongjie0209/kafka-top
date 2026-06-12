use crate::kafka::client::KafkaClient;
use crate::state::*;
use anyhow::Result;
use rdkafka::consumer::Consumer;
use rdkafka::metadata::Metadata;
use std::time::Duration;

fn fetch_metadata(client: &KafkaClient) -> Result<Metadata> {
    client
        .admin_client
        .inner()
        .fetch_metadata(None, Duration::from_secs(10))
        .map_err(|e| anyhow::anyhow!("Failed to fetch metadata: {}", e))
}

pub fn fetch_dashboard_data(client: &KafkaClient) -> Result<DashboardData> {
    let metadata = fetch_metadata(client)?;

    let connected_broker = format!("{} (id {})", metadata.orig_broker_name(), metadata.orig_broker_id());
    let broker_count = metadata.brokers().len();
    let brokers: Vec<BrokerInfo> = metadata
        .brokers()
        .iter()
        .map(|b| BrokerInfo {
            id: b.id(),
            host: b.host().to_string(),
            port: b.port(),
        })
        .collect();

    let topic_count = metadata
        .topics()
        .iter()
        .filter(|t| !t.name().starts_with('_'))
        .count();

    let group_count = client
        .consumer
        .fetch_group_list(None, Duration::from_secs(10))
        .map(|groups| groups.groups().len())
        .unwrap_or(0);

    Ok(DashboardData {
        connected_broker,
        broker_count,
        topic_count,
        group_count,
        brokers,
    })
}

pub fn fetch_topics(client: &KafkaClient, filter: Option<&str>) -> Result<TopicsData> {
    let metadata = fetch_metadata(client)?;

    let topics: Vec<TopicSummary> = metadata
        .topics()
        .iter()
        .filter(|t| !t.name().starts_with('_'))
        .filter(|t| {
            filter.map_or(true, |f| t.name().to_lowercase().contains(&f.to_lowercase()))
        })
        .map(|t| {
            let total_messages: i64 = t
                .partitions()
                .iter()
                .map(|p| {
                    client
                        .consumer
                        .fetch_watermarks(t.name(), p.id(), Duration::from_secs(5))
                        .map(|(_low, high)| high)
                        .unwrap_or(0)
                })
                .sum();

            TopicSummary {
                name: t.name().to_string(),
                partition_count: t.partitions().len(),
                total_messages,
            }
        })
        .collect();

    Ok(TopicsData { topics })
}

pub fn fetch_topic_detail(client: &KafkaClient, topic: &str) -> Result<TopicDetailData> {
    let metadata = fetch_metadata(client)?;

    let topic_metadata = metadata
        .topics()
        .iter()
        .find(|t| t.name() == topic)
        .ok_or_else(|| anyhow::anyhow!("Topic '{}' not found", topic))?;

    let partitions: Vec<PartitionInfo> = topic_metadata
        .partitions()
        .iter()
        .map(|p| {
            let (_low, high) = client
                .consumer
                .fetch_watermarks(topic, p.id(), Duration::from_secs(5))
                .unwrap_or((-1, -1));

            PartitionInfo {
                id: p.id(),
                leader: p.leader(),
                replicas: p.replicas().to_vec(),
                isr: p.isr().to_vec(),
                log_end_offset: high,
                producer_rate: 0.0,
            }
        })
        .collect();

    Ok(TopicDetailData {
        topic: topic.to_string(),
        partitions,
    })
}
