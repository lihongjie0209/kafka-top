#[derive(Debug, Clone)]
pub struct TopicSummary {
    pub name: String,
    pub partition_count: usize,
    pub total_messages: i64,
}

#[derive(Debug, Clone)]
pub struct TopicsData {
    pub topics: Vec<TopicSummary>,
}

#[derive(Debug, Clone)]
pub struct PartitionInfo {
    pub id: i32,
    pub leader: i32,
    pub replicas: Vec<i32>,
    pub isr: Vec<i32>,
    pub log_end_offset: i64,
    pub producer_rate: f64,
}

#[derive(Debug, Clone)]
pub struct TopicDetailData {
    pub topic: String,
    pub partitions: Vec<PartitionInfo>,
}
