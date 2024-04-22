use scylla::{Session, SessionBuilder, transport::errors::QueryError};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Person {
    name: String,
    age: u32,
}

#[tokio::main]
async fn main() -> Result<(), QueryError> {
    let uri = "127.0.0.1:9042"; // Replace if your ScyllaDB is on a different IP
    let keyspace = "my_keyspace"; // Replace with your keyspace name

    let session: Session = SessionBuilder::new()
        .known_node(uri)
        .build()
        .await
        .expect("failed to connect to ScyllaDB");

    // You'll need to create a table in ScyllaDB beforehand, e.g.:
    // CREATE TABLE my_keyspace.people (name text, age int, PRIMARY KEY(name));

    let person = Person {
        name: "Alice".to_string(),
        age: 30,
    };

    let json_value = serde_json::to_string(&person).expect("failed to Serialize Person to JSON format");

    session.query(format!("INSERT INTO {}.people JSON ?", keyspace), (json_value,))
        .await?;

    Ok(())
}