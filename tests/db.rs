use std::io::Write;
use std::process::{Command, Output};

// start local db from `dbs.yml`
async fn start_db(file: &str, db: &str) -> anyhow::Result<Output> {
    let output = Command::new("docker-compose")
        .args(&["-f", file, "up", "-d", db])
        .output()?;
    std::io::stdout().write_all(&output.stdout)?;
    std::io::stderr().write_all(&output.stderr)?;
    Ok(output)
}

// stop local db from `dbs.yml`
async fn stop_db(db: &str) -> anyhow::Result<Output> {
    let output = Command::new("docker-compose")
        .args(&["stop", db])
        .output()?;
    std::io::stdout().write_all(&output.stdout)?;
    std::io::stderr().write_all(&output.stderr)?;
    Ok(output)
}

#[tokio::test]
async fn postgres() {
    let output = start_db("./tests/dbs.yml", "postgres").await.expect("Failed to start PostgreSQL service");
    assert!(output.status.success(), "Failed to start PostgreSQL service");
    println!("PostgreSQL service started successfully.");

    // ping postgres
    // insert doc
    // remove doc

    let output = stop_db("postgres").await.expect("Failed to stop PostgreSQL service");
    assert!(output.status.success(), "Failed to stop PostgreSQL service");
    println!("PostgreSQL service stopped successfully.");
}

#[tokio::test]
async fn couchdb() {
    let output = start_db("./tests/dbs.yml", "couchdb").await.expect("Failed to start CouchDB service");
    assert!(output.status.success(), "Failed to start CouchDB service");
    println!("CouchDB service started successfully.");

    // ping couch
    // insert doc
    // remove doc

    let output = stop_db("couchdb").await.expect("Failed to stop CouchDB service");
    assert!(output.status.success(), "Failed to stop CouchDB service");
    println!("CouchDB service stopped successfully.");
}

#[tokio::test]
async fn scylladb() {
    let output = start_db("./tests/dbs.yml", "scylla").await.expect("Failed to start ScyllaDB service");
    assert!(output.status.success(), "Failed to start ScyllaDB service");
    println!("ScyllaDB service started successfully.");

    // ping scylla
    // insert doc
    // remove doc

    let output = stop_db("scylla").await.expect("Failed to stop ScyllaDB service");
    assert!(output.status.success(), "Failed to stop ScyllaDB service");
    println!("ScyllaDB service stopped successfully.");
}