#[derive(Debug, Clone)]
pub struct BrokerInfo {
    pub id: i32,
    pub host: String,
    pub port: i32,
}

#[derive(Debug, Clone)]
pub struct DashboardData {
    pub connected_broker: String,
    pub broker_count: usize,
    pub topic_count: usize,
    pub group_count: usize,
    pub brokers: Vec<BrokerInfo>,
}
