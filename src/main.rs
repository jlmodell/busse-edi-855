use std::error::Error;
use futures::stream::TryStreamExt;
use serde::{Deserialize, Serialize};
use env_file_reader::read_file;
use tokio;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct EndPoint {
    name: String,
    uri: String,
}

async fn get_all_endpoints(c: mongodb::Collection<EndPoint>) -> Vec<EndPoint> {    
    let cursor = match c.find(None, None).await {
        Ok(cursor) => cursor,
        Err(_) => return vec![]
    };
    
    cursor.try_collect().await.unwrap_or_else(|_| vec![])
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let env_vars: std::collections::HashMap<String, String> = read_file(".env")?;

    let mongodb_uri: String = env_vars.get("MONGODB_URI").unwrap().to_owned();

    let options: mongodb::options::ClientOptions = mongodb::options::ClientOptions::parse(mongodb_uri).await?;
    
    let client: mongodb::Client = mongodb::Client::with_options(options)?;

    println!("Databases:");
    for name in client.list_database_names(None, None).await? {
        println!("- {}", name);
    }

    let endpoints: mongodb::Collection<EndPoint> = client.database("busse_shopfloor").collection("endpoints");

    let endpoints: Vec<EndPoint> = get_all_endpoints(endpoints).await;

    for endpoint in endpoints {
        println!("{:?}", endpoint);
    }
    
    Ok(())
}

