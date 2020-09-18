use reqwest;
use crate::akka_cluster::model::ClusterStatus;

pub fn get_akka_cluster_status(url: &String) -> Result<ClusterStatus, String> {
    get_akka_cluster_status_async(url)
}

#[tokio::main]
async fn get_akka_cluster_status_async(url: &String) -> Result<ClusterStatus, String> {
    let response = reqwest::get(url).await.map_err(|e| e.to_string())?;
    if response.status().is_success() {
        let cluster_status: ClusterStatus = response.json().await.map_err(|e| e.to_string())?;
        Ok(cluster_status)
    } else {
        Err(format!("Request to get cluster status failed with status {}", response.status()))
    }
}
