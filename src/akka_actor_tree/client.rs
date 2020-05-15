use reqwest;
use serde_json::Value;
use serde::Deserialize;
use crate::akka_actor_tree::model::ActorNode;
use std::collections::HashMap;

pub fn get_actors(url: &String, timeout: u64) -> Result<Vec<ActorNode>, String> {
    get_actors_async(url, timeout)
}

pub fn get_actor_count(url: &String, timeout: u64) -> Result<u64, String> {
    get_actor_count_async(url, timeout)
}

#[tokio::main]
async fn get_actors_async(url: &String, timeout: u64) -> Result<Vec<ActorNode>, String> {
    let url = format!("{}?timeout={}", url, timeout);
    let response = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("Request to get actor tree failed with status: {}", response.status()));
    }

    let mut response_body: HashMap<String, Value> = response.json().await.map_err(|e| e.to_string())?;
    Ok(build_actor_tree(&mut response_body))
}

fn build_actor_tree(json: &mut HashMap<String, Value>) -> Vec<ActorNode> {
    let mut actors: Vec<ActorNode> = vec![];
    // user actors should go first
    if let Some(v) = json.get("user") {
        actors.push(ActorNode { name: "user".to_string(), parent: None, id: 1 });
        build_actor_tree_iter(&v, Some(1), &mut actors)
    }

    for (k, v) in json {
        if k != "user" {
            let id = actors.len() + 1;
            actors.push(ActorNode { name: k.to_owned(), parent: None, id });
            build_actor_tree_iter(&v, Some(id), &mut actors)
        }
    }
    actors
}

fn build_actor_tree_iter(json: &Value, parent_id: Option<usize>, actors: &mut Vec<ActorNode>) {
    if let Value::Object(mm) = json {
        for (k, v) in mm {
            let id = actors.len() + 1;
            actors.push(ActorNode { name: k.to_owned(), parent: parent_id, id });
            build_actor_tree_iter(&v, Some(id), actors);
        }
    };
}

#[derive(Deserialize)]
struct CountResult {
    result: u64
}

#[tokio::main]
async fn get_actor_count_async(url: &String, timeout: u64) -> Result<u64, String> {
    let url = format!("{}?timeout={}", url, timeout);
    let response = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("Request to get actor count failed with status {}", response.status()));
    }
    let body: CountResult = response.json().await.map_err(|e| e.to_string())?;
    Ok(body.result)
}
