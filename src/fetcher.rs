use jmx::MBeanClient;

use crate::akka;
use crate::akka::model::{ActorTreeNode, AkkaSettings, DeadLettersSnapshot, DeadLettersWindow, ActorSystemStatus, ClusterMember};
use crate::jmx::client::JMXClient;
use crate::jmx::model::{HikariMetrics, JMXConnectionSettings, SlickConfig, SlickMetrics};
use crate::zio::model::Fiber;
use crate::zio::zmx::{NetworkZMXClient, ZMXClient};

pub enum FetcherRequest {
    FiberDump,
    RegularFiberDump,
    HikariMetrics,
    SlickMetrics,
    SlickConfig,
    ActorTree,
    ActorSystemStatus,
    DeadLetters,
    ClusterStatus,
}

pub enum FetcherResponse {
    FiberDump(Result<Vec<Fiber>, String>),
    RegularFiberDump(Result<Vec<Fiber>, String>),
    HikariMetrics(Result<HikariMetrics, String>),
    SlickMetrics(Result<SlickMetrics, String>),
    SlickConfig(Result<SlickConfig, String>),
    ActorTree(Result<Vec<ActorTreeNode>, String>),
    ActorSystemStatus(Result<ActorSystemStatus, String>),
    DeadLetters(Result<(DeadLettersSnapshot, DeadLettersWindow), String>),
    ClusterStatus(Result<Vec<ClusterMember>, String>),
    FatalFailure(String),
}

pub struct Fetcher {
    pub zmx_client: Option<Box<dyn ZMXClient>>,
    pub jmx: Option<JMXClient>,
    pub akka_settings: Option<AkkaSettings>,
}

impl Fetcher {
    pub fn new(
        zio_zmx_addr: Option<String>,
        jmx: Option<JMXConnectionSettings>,
        akka: Option<AkkaSettings>) -> Result<Fetcher, String> {
        let jmx_client: Option<JMXClient> = match jmx {
            None => Ok(None),
            Some(conn) => {
                let url_str = format!(
                    "service:jmx:rmi://{}/jndi/rmi://{}/jmxrmi",
                    &conn.address, &conn.address
                );
                let url = jmx::MBeanAddress::service_url(url_str.clone());
                MBeanClient::connect(url)
                    .map(|x| Some(JMXClient::new(x, conn.db_pool_name.clone())))
                    .map_err(|e| format!(
                        "Couldn't connect to jmx at {}. Error: {}", url_str, e
                    ))
            }
        }?;

        Ok(Fetcher {
            zmx_client: zio_zmx_addr.map(|x| {
                let a: Box<dyn ZMXClient> = Box::new(NetworkZMXClient::new(x));
                a
            }),
            jmx: jmx_client,
            akka_settings: akka,
        })
    }

    pub fn dump_fibers(&self) -> Result<Vec<Fiber>, String> {
        self.zmx_client.as_ref().unwrap().dump_fibers()
            .map_err(
                |e| format!(
                    "Couldn't get fiber dump from {}. Make sure zio-zmx is listening on specified port. Underlying error: {}",
                    self.zmx_client.as_ref().unwrap().address(),
                    e
                )
            )
    }

    pub fn get_hikari_metrics(&self) -> Result<HikariMetrics, String> {
        self.jmx.as_ref().unwrap().get_hikari_metrics().map_err(|e| Fetcher::format_slick_error(e))
    }

    pub fn get_slick_metrics(&self) -> Result<SlickMetrics, String> {
        self.jmx.as_ref().unwrap().get_slick_metrics().map_err(|e| Fetcher::format_slick_error(e))
    }

    pub fn get_slick_config(&self) -> Result<SlickConfig, String> {
        self.jmx.as_ref().unwrap().get_slick_config().map_err(|e| Fetcher::format_slick_error(e))
    }

    pub fn get_actor_tree(&self) -> Result<Vec<ActorTreeNode>, String> {
        let s = self.akka_settings.as_ref().unwrap();
        akka::client::get_actors(&s.tree_address, s.tree_timeout)
            .map_err(|e| format!("Error loading akka actor tree tree: {}", e))
    }

    pub fn get_actor_system_status(&self) -> Result<ActorSystemStatus, String> {
        let s = self.akka_settings.as_ref().unwrap();
        akka::client::get_actor_system_status(&s.status_address, s.status_timeout)
            .map_err(|e| format!("Error loading akka actor system status: {}", e))
    }

    pub fn get_akka_cluster_status(&self) -> Result<Vec<ClusterMember>, String> {
        let s = self.akka_settings.as_ref().unwrap();
        akka::client::get_akka_cluster_status(s.cluster_status_address.as_ref().unwrap())
    }

    pub fn get_dead_letters(&self) -> Result<(DeadLettersSnapshot, DeadLettersWindow), String> {
        let s = self.akka_settings.as_ref().unwrap();
        akka::client::get_deadletters(&s.dead_letters_address, s.dead_letters_window)
            .map_err(|e| format!("Error loading dead letters metrics: {}", e))
    }

    fn format_slick_error(e: jmx::Error) -> String {
        format!(
            "No Slick JMX metrics found. Are you sure you have registerMbeans=true in your Slick config?\r\nUnderlying error: {}", e
        )
    }
}
