// each test runs a contained database, then connects and pings it
// inserting a doc, removing a doc; to be added

#[tokio::test]
async fn couchdb() {
    // start couchdb
    let output = start_db("./tests/dbs.yml", "couchdb").await.expect("Failed to start CouchDB service");
    assert!(output.status.success(), "Failed to start CouchDB service");
    println!("CouchDB service started successfully.");

    // wait for db to initialise
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // conn
    let conn = "http://admin:password@localhost:5984";
    let client = reqwest::Client::new();

    // ping
    let response = client
        .get(conn)
        .send()
        .await
        .expect("failed to retrieve GET response");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    println!("Ping successful.");

    // insert doc
    // use pipe_io::db::couchdb::insert_doc;
    // remove doc

    // stop couchdb
    let output = stop_db("couch-test").await.expect("Failed to stop CouchDB service");
    assert!(output.status.success(), "Failed to stop CouchDB service");
    println!("CouchDB service stopped successfully.");
}

#[tokio::test]
async fn postgresql() {
    // start postgresql
    let output = start_db("./tests/dbs.yml", "postgres").await.expect("Failed to start PostgreSQL service");
    assert!(output.status.success(), "Failed to start PostgreSQL service");
    println!("PostgreSQL service started successfully.");

    // wait for db to initialise
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // conn
    let (client, connection) = 
        tokio_postgres::connect("host=/var/lib/postgresql,localhost user=postgres password=password port=5432", tokio_postgres::NoTls)
            .await
            .expect("Failed to connect to PostgreSQL service");
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // ping (simple query)
    let rows = client
        .query("SELECT $1::TEXT", &[&"hello world"])
        .await
        .expect("Failed to query PostgreSQL service");
    let value: &str = rows[0].get(0);
    assert_eq!(value, "hello world");
    println!("Ping successful.");

    // insert doc
    // remove doc

    // stop postgresql
    let output = stop_db("postgres-test").await.expect("Failed to stop PostgreSQL service");
    assert!(output.status.success(), "Failed to stop PostgreSQL service");
    println!("PostgreSQL service stopped successfully.");
}

#[tokio::test]
async fn scylladb() {
    // start scylladb
    let output = start_db("./tests/dbs.yml", "scylla").await.expect("Failed to start ScyllaDB service");
    assert!(output.status.success(), "Failed to start ScyllaDB service");
    println!("ScyllaDB service started successfully.");

    // wait for db to initialise
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    // conn
    let session: scylla::Session = scylla::SessionBuilder::new()
        .known_node("127.0.0.1:9042")
        .compression(Some(scylla::transport::Compression::Lz4))
        .build()
        .await
        .expect("Failed to connect to ScyllaDB service");

    // ping
    let result = session.query("SELECT now()", &[]).await;
    println!("{:#?}", result);
    println!("Ping successful.");
    // assert!(result.is_ok());

    // insert doc
    // remove doc

    // stop scylladb
    let output = stop_db("scylla-test").await.expect("Failed to stop ScyllaDB service");
    assert!(output.status.success(), "Failed to stop ScyllaDB service");
    println!("ScyllaDB service stopped successfully.");
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// util
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// start local db from some yaml_file_path
// `docker-compose -f <file> up -d`
async fn start_db(file: &str, container: &str) -> anyhow::Result<std::process::Output> {
    let output = std::process::Command::new("docker-compose")
        .args(&["-f", file, "up", "-d", container])
        .output()?;
    Ok(output)
}

// stop local db; container has to be named
// `docker stop <container>`
async fn stop_db(container: &str) -> anyhow::Result<std::process::Output> {
    let output = std::process::Command::new("docker")
        .args(&["stop", container])
        .output()?;
    Ok(output)
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// datasets
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

static _EXAMPLE_JSON: &str = r#"
    {
        "hello": "world"
    }
"#;

#[derive(serde::Deserialize, serde::Serialize)]
struct ExampleJson {
    hello: String,
}