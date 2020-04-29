#[derive(Clone)]
pub struct JMXConnectionSettings {
    pub address: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub db_pool_name: String,
}

#[derive(Clone)]
pub struct SlickMetrics {
    pub active_threads: i32,
    pub queue_size: i32,
}

impl SlickMetrics {
    pub const ZERO: SlickMetrics = SlickMetrics {
        active_threads: 0,
        queue_size: 0,
    };
}

pub struct SlickConfig {
    pub max_threads: i32,
    pub max_queue_size: i32,
}

#[derive(Clone)]
pub struct HikariMetrics {
    pub total: i32,
    pub active: i32,
    pub idle: i32,
    pub waiting: i32,
}
