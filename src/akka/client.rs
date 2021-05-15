use reqwest;
use serde_json::Value;
use crate::akka::model::*;
use std::collections::HashMap;

pub fn get_actors(url: &String, timeout: u64) -> Result<Vec<ActorTreeNode>, String> {
    get_actors_async(url, timeout)
}

pub fn get_actor_system_status(url: &String, timeout: u64) -> Result<ActorSystemStatus, String> {
    get_actor_system_status_async(url, timeout)
}

pub fn get_deadletters(url: &String, window: u64) -> Result<(DeadLettersSnapshot, DeadLettersWindow), String> {
    get_deadletters_async(url, window)
}

#[tokio::main]
async fn get_deadletters_async(url: &String, window: u64) -> Result<(DeadLettersSnapshot, DeadLettersWindow), String> {
    let url = format!("{}?window={}", url, window);
    let response = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    if response.status().is_success() {
        let metrics: DeadLettersMetrics = response.json().await.map_err(|e| e.to_string())?;
        Ok((metrics.snapshot, metrics.window))
    } else {
        Err(format!("Request to get actor tree failed with status: {}", response.status()))
    }
}

#[tokio::main]
async fn get_actors_async(url: &String, timeout: u64) -> Result<Vec<ActorTreeNode>, String> {
    let url = format!("{}?timeout={}", url, timeout);
    let response = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    if response.status().is_success() {
        let response_body: HashMap<String, Value> = response.json().await.map_err(|e| e.to_string())?;
        Ok(build_actor_tree(response_body))
    } else {
        Err(format!("Request to get actor tree failed with status: {}", response.status()))
    }
}

fn build_actor_tree(json: HashMap<String, Value>) -> Vec<ActorTreeNode> {
    let mut actors: Vec<ActorTreeNode> = vec![];
    // user actors should go first
    if let Some(v) = json.get("user") {
        actors.push(ActorTreeNode { name: "user".to_owned(), parent: None, id: 1 });
        build_actor_tree_iter(v, Some(1), &mut actors)
    }

    for (k, v) in json {
        if k != "user" {
            let id = actors.len() + 1;
            actors.push(ActorTreeNode { name: k, parent: None, id });
            build_actor_tree_iter(&v, Some(id), &mut actors)
        }
    }
    actors
}

fn build_actor_tree_iter(json: &Value, parent_id: Option<usize>, actors: &mut Vec<ActorTreeNode>) {
    if let Value::Object(mm) = json {
        for (k, v) in mm {
            let id = actors.len() + 1;
            actors.push(ActorTreeNode { name: k.to_owned(), parent: parent_id, id });
            build_actor_tree_iter(&v, Some(id), actors);
        }
    };
}

#[tokio::main]
async fn get_actor_system_status_async(url: &String, timeout: u64) -> Result<ActorSystemStatus, String> {
    let url = format!("{}?timeout={}", url, timeout);
    let response = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    if response.status().is_success() {
        let body: ActorSystemStatus = response.json().await.map_err(|e| e.to_string())?;
        Ok(body)
    } else {
        Err(format!("Request to get actor count failed with status {}", response.status()))
    }
}
