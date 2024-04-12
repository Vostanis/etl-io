use super::{db::*, Error};

/// Fetch data from some endpoint, also known as `path`);
/// default implementation assumes `&str` input type, resembling either a File Path or a URL.
pub async fn extract<I>(path: &str) -> Result<I, Error>
where
    I: serde::de::DeserializeOwned + Send,
{
    if path.starts_with("http") {
        extract_url(path).await
    } else {
        extract_file(path).await
    }
}

/// Reads a JSON file and deserializes to some `I` type.
pub async fn extract_file<I>(file_path: &str) -> Result<I, Error>
where
    I: serde::de::DeserializeOwned + Send,
{
    let file = std::fs::File::open(file_path)?;
    let reader = std::io::BufReader::new(file);
    let data = serde_json::from_reader(reader)?;
    Ok(data)
}

/// GET request a URL (with a client), deserializing the JSON response to some `I` type.
///
/// Provides 1 (very cheeky & anonymous) HTTP header:
///
/// `{ "User-Agent":"example@example.com" }`
pub async fn extract_url<I>(url: &str) -> Result<I, Error>
where
    I: serde::de::DeserializeOwned + Send,
{
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("User-Agent", "example@example.com")
        .send()
        .await?
        .text()
        .await?;
    let data: I = serde_json::from_str(&response)?;
    Ok(data)
}

/// Load to a database.
/// The default implementation has a list of current database APIs:
/// - CouchDB
// - ScyllaDB
// - PostgreSQL
///
/// The assumed workflow is: take a file/table and create/update it
pub async fn load<O>(output: O, conn: &str, doc_id: &str) -> Result<(), Error>
where
    O: for<'a> serde::de::Deserialize<'a> + serde::Serialize + Send,
{
    let _ = load_couchdb(output, conn, doc_id).await;
    Ok(())
}

/// Loads document to CouchDB.
pub async fn load_couchdb<O>(output: O, conn: &str, doc_id: &str) -> Result<(), Error>
where
    O: for<'a> serde::de::Deserialize<'a> + serde::Serialize + Send,
{
    couchdb::insert_doc::<O>(&output, conn, doc_id).await;
    Ok(())
}
