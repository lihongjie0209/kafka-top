use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "kafka-top", about = "Kafka cluster monitoring TUI")]
pub struct Cli {
    /// Kafka bootstrap servers (comma-separated)
    #[arg(short, long, default_value = "localhost:9092", env = "KAFKA_BOOTSTRAP_SERVERS")]
    pub bootstrap_servers: String,

    /// Filter topics by name (substring match)
    #[arg(short, long)]
    pub topic_filter: Option<String>,

    /// Filter consumer groups by name (substring match)
    #[arg(short, long)]
    pub group_filter: Option<String>,

    /// Auto-refresh interval in seconds
    #[arg(short, long, default_value = "5")]
    pub refresh_interval: u64,

    /// Extra Kafka client config options (key=value)
    #[arg(short = 'C', long = "kafka-config", value_name = "KEY=VALUE")]
    pub kafka_config: Vec<String>,
}
