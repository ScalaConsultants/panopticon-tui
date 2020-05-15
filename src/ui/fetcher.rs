use jmx::MBeanClient;
use crate::akka_actor_tree;
use crate::akka_actor_tree::model::{ActorNode, AkkaActorTreeSettings};
use crate::jmx_client::client::JMXClient;
use crate::jmx_client::model::{HikariMetrics, JMXConnectionSettings, SlickConfig, SlickMetrics};
use crate::zio::model::Fiber;
use crate::zio::zmx_client::{NetworkZMXClient, ZMXClient};

pub enum FetcherRequest {
    FiberDump,
    RegularFiberDump,
    HikariMetrics,
    SlickMetrics,
    SlickConfig,
    ActorTree,
    ActorCount,
}

pub enum FetcherResponse {
    FiberDump(Result<Vec<Fiber>, String>),
    RegularFiberDump(Result<Vec<Fiber>, String>),
    HikariMetrics(Result<HikariMetrics, String>),
    SlickMetrics(Result<SlickMetrics, String>),
    SlickConfig(Result<SlickConfig, String>),
    ActorTree(Result<Vec<ActorNode>, String>),
    ActorCount(Result<u64, String>),
    FatalFailure(String),
}

pub struct Fetcher {
    pub zmx_client: Option<Box<dyn ZMXClient>>,
    pub jmx: Option<JMXClient>,
    pub actor_tree_settings: Option<AkkaActorTreeSettings>,
}

impl Fetcher {
    pub fn new(
        zio_zmx_addr: Option<String>,
        jmx: Option<JMXConnectionSettings>,
        akka_actor_tree: Option<AkkaActorTreeSettings>) -> Result<Fetcher, String> {
        let jmx_client: Option<JMXClient> = match jmx {
            None => Ok(None),
            Some(conn) => {
                let url_str = format!(
                    "service:jmx:rmi://{}/jndi/rmi://{}/jmxrmi",
                    &conn.address, &conn.address
                );
                let url = jmx::MBeanAddress::service_url(url_str.clone());
                eprintln!("Connecting...");
                let r = MBeanClient::connect(url)
                    .map(|x| Some(JMXClient::new(x, conn.db_pool_name.clone())))
                    .map_err(|e| format!(
                        "Couldn't connect to jmx at {}. Error: {}", url_str, e
                    ));
                eprintln!("Connected...");
                r
            }
        }?;

        Ok(Fetcher {
            zmx_client: zio_zmx_addr.map(|x| {
                let a: Box<dyn ZMXClient> = Box::new(NetworkZMXClient::new(x));
                a
            }),
            jmx: jmx_client,
            actor_tree_settings: akka_actor_tree,
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

    pub fn get_actor_tree(&self) -> Result<Vec<ActorNode>, String> {
        let s = self.actor_tree_settings.as_ref().unwrap();
        akka_actor_tree::client::get_actors(&s.tree_address, s.tree_timeout)
            .map_err(|e| format!("Error loading akka actor tree tree: {}", e))
    }

    pub fn get_actor_count(&self) -> Result<u64, String> {
        let s = self.actor_tree_settings.as_ref().unwrap();
        akka_actor_tree::client::get_actor_count(&s.count_address, s.count_timeout)
            .map_err(|e| format!("Error loading akka actor count: {}", e))
    }

    fn format_slick_error(e: jmx::Error) -> String {
        format!(
            "No Slick JMX metrics found. Are you sure you have registerMbeans=true in your Slick config?\r\nUnderlying error: {}", e
        )
    }
}
