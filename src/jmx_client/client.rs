use crate::jmx_client::model::*;
use jmx::{MBeanClient, MBeanClientTrait};
use serde::de::DeserializeOwned;

pub struct JMXClient {
    pub connection: MBeanClient,
    pub db_pool_name: String,
}

impl JMXClient {
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
}


