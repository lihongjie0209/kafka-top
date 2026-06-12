#[derive(Debug, Clone)]
pub struct GroupSummary {
    pub group_id: String,
    pub state: String,
    pub protocol_type: String,
    pub member_count: i32,
}

#[derive(Debug, Clone)]
pub struct GroupsData {
    pub groups: Vec<GroupSummary>,
}

#[derive(Debug, Clone)]
pub struct PartitionAssignment {
    pub topic: String,
    pub partition_id: i32,
    pub current_offset: i64,
    pub log_end_offset: i64,
    pub lag: i64,
    /// Consumer rate in messages/sec (0 if first measurement)
    pub consumer_rate: f64,
}

#[derive(Debug, Clone)]
pub struct GroupMemberInfo {
    pub member_id: String,
    pub client_id: String,
    pub client_host: String,
}

#[derive(Debug, Clone)]
pub struct GroupDetailData {
    pub group_id: String,
    pub state: String,
    pub protocol: String,
    pub members: Vec<GroupMemberInfo>,
    pub assignments: Vec<PartitionAssignment>,
}
