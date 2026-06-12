use crate::kafka::client::KafkaClient;
use crate::state::*;
use anyhow::Result;
use rdkafka::consumer::Consumer;
use rdkafka::topic_partition_list::{Offset, TopicPartitionList};
use std::time::Duration;

/// Fetch consumer group list
pub fn fetch_groups(client: &KafkaClient, filter: Option<&str>) -> Result<GroupsData> {
    let gl = client
        .consumer
        .fetch_group_list(None, Duration::from_secs(10))
        .map_err(|e| anyhow::anyhow!("Failed to list consumer groups: {}", e))?;

    let groups: Vec<GroupSummary> = gl
        .groups()
        .iter()
        .filter(|g| {
            filter.map_or(true, |f| {
                g.name().to_lowercase().contains(&f.to_lowercase())
            })
        })
        .map(|g| GroupSummary {
            group_id: g.name().to_string(),
            state: g.state().to_string(),
            protocol_type: g.protocol_type().to_string(),
            member_count: g.members().len() as i32,
        })
        .collect();

    Ok(GroupsData { groups })
}

/// Fetch group detail with member info and lag
pub fn fetch_group_detail(client: &KafkaClient, group_id: &str) -> Result<GroupDetailData> {
    let gl = client
        .consumer
        .fetch_group_list(Some(group_id), Duration::from_secs(10))
        .map_err(|e| anyhow::anyhow!("Failed to fetch group '{}': {}", group_id, e))?;

    let group_info = gl
        .groups()
        .iter()
        .find(|g| g.name() == group_id)
        .ok_or_else(|| anyhow::anyhow!("Consumer group '{}' not found", group_id))?;

    let state = group_info.state().to_string();
    let protocol = group_info.protocol().to_string();

    let members: Vec<GroupMemberInfo> = group_info
        .members()
        .iter()
        .map(|m| GroupMemberInfo {
            member_id: m.id().to_string(),
            client_id: m.client_id().to_string(),
            client_host: m.client_host().to_string(),
        })
        .collect();

    // Fetch committed offsets: iterate all topics to build a TPL
    let metadata = client
        .admin_client
        .inner()
        .fetch_metadata(None, Duration::from_secs(10))
        .map_err(|e| anyhow::anyhow!("Failed to fetch metadata: {}", e))?;

    let mut tpl = TopicPartitionList::new();
    for topic in metadata.topics() {
        if topic.name().starts_with('_') {
            continue;
        }
        for partition in topic.partitions() {
            tpl.add_partition(topic.name(), partition.id());
        }
    }

    let mut assignments = Vec::new();
    if !tpl.elements().is_empty() {
        match client.consumer.committed_offsets(tpl, Duration::from_secs(10)) {
            Ok(committed_tpl) => {
                for elem in committed_tpl.elements() {
                    let offset_val = match elem.offset() {
                        Offset::Offset(o) if o >= 0 => o,
                        _ => continue,
                    };

                    let (_low, log_end) = client
                        .consumer
                        .fetch_watermarks(elem.topic(), elem.partition(), Duration::from_secs(5))
                        .unwrap_or((-1, -1));

                    let lag = if log_end >= 0 {
                        (log_end - offset_val).max(0)
                    } else {
                        -1
                    };

                    assignments.push(PartitionAssignment {
                        topic: elem.topic().to_string(),
                        partition_id: elem.partition(),
                        current_offset: offset_val,
                        log_end_offset: log_end,
                        lag,
                        consumer_rate: 0.0,
                    });
                }
            }
            Err(e) => {
                // If committed_offsets fails (e.g., the group has no committed offsets yet),
                // just return empty assignments
                tracing::warn!(
                    "Failed to fetch committed offsets for group '{}': {}",
                    group_id,
                    e
                );
            }
        }
    }

    Ok(GroupDetailData {
        group_id: group_id.to_string(),
        state,
        protocol,
        members,
        assignments,
    })
}
