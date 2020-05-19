use crate::jmx::model::*;
use jmx::{MBeanClient, MBeanClientTrait};
use serde::de::DeserializeOwned;

pub struct JMXClient {
    connection: MBeanClient,
    db_pool_name: String,
}

impl JMXClient {
    pub fn new(connection: MBeanClient, db_pool_name: String) -> JMXClient {
        JMXClient { connection, db_pool_name }
    }

    pub fn get_hikari_metrics(&self) -> Result<HikariMetrics, jmx::Error> {
        let total: i32 = self.get_hikari_attribute("TotalConnections")?;
        let active: i32 = self.get_hikari_attribute("ActiveConnections")?;
        let waiting: i32 = self.get_hikari_attribute("ThreadsAwaitingConnection")?;
        let idle: i32 = self.get_hikari_attribute("IdleConnections")?;

        Result::Ok(HikariMetrics {
            total,
            active,
            waiting,
            idle,
        })
    }

    pub fn get_slick_metrics(&self) -> Result<SlickMetrics, jmx::Error> {
        let active_threads: i32 = self.get_slick_attribute("ActiveThreads")?;
        let queue_size: i32 = self.get_slick_attribute("QueueSize")?;

        Result::Ok(SlickMetrics {
            active_threads,
            queue_size,
        })
    }

    pub fn get_slick_config(&self) -> Result<SlickConfig, jmx::Error> {
        let max_threads: i32 = self.get_slick_attribute("MaxThreads")?;
        let max_queue_size: i32 = self.get_slick_attribute("MaxQueueSize")?;

        Result::Ok(SlickConfig {
            max_threads,
            max_queue_size,
        })
    }

    fn get_slick_attribute<T: DeserializeOwned>(&self, attr: &str) -> Result<T, jmx::Error> {
        self.connection.get_attribute(format!("slick:type=AsyncExecutor,name={}", self.db_pool_name), attr)
    }

    fn get_hikari_attribute<T: DeserializeOwned>(&self, attr: &str) -> Result<T, jmx::Error> {
        self.connection.get_attribute(format!("com.zaxxer.hikari:type=Pool ({})", self.db_pool_name), attr)
    }
}


