use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct CouchDocument {
    _id: String,
    _rev: String,
}

/// Deploys a [`reqwest Client`].
///
/// [`reqwest Client`]: (https://docs.rs/reqwest/latest/reqwest/struct.Client.html)
///
/// Initially, the client sends a GET request to the database and awaits the response.
/// One of two responses will elicit further actions (any other response will panic):
///
/// - `Status Code: OK`; the file alreadys exists, so we update it by retrieving the Revision ID (_rev) and PUTing
/// the file up with this new ID.
/// - Status: NOT_FOUND; the file does not exist, so we then PUT the document with an empty Revision ID.
///
/// See the [`CouchDB Documentation`]  for more details.
///
/// [`CouchDB Documentation`]: (https://docs.couchdb.org/en/stable/intro/index.html)
pub async fn insert_doc<T>(data: &T, conn: &str, doc_id: &str)
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    // check if the document already exists with a GET request
    let conn = format!("{conn}/{doc_id}");
    let client = reqwest::Client::new();
    let response = client
        .get(conn.clone())
        .send()
        .await
        .expect("failed to retrieve GET response");

    match response.status() {
        // "if the file already exists ..."
        reqwest::StatusCode::OK => {
            // retrieve current Revision ID
            let text = response
                .text()
                .await
                .expect("failed to translate response to text");
            let current_file: CouchDocument = serde_json::from_str(&text)
                .expect("failed to read current revision to serde schema");

            // PUT the file up with current Revision ID
            let mut updated_file = json!(data);
            updated_file["_rev"] = json!(current_file._rev);
            let _second_response = client
                .put(conn)
                .json(&updated_file)
                .send()
                .await
                .expect("failed to retrieve PUT response");
        }

        // "if the file does not exist ..."
        reqwest::StatusCode::NOT_FOUND => {
            // PUT the new file up, requiring no Revision ID (where we use an empty string)
            let new_file = json!(data);
            let _second_response = client
                .put(conn)
                .json(&new_file)
                .send()
                .await
                .expect("failed to retrieve PUT response");
        }

        _ => eprintln!("unexpected status code found - expected `OK` or `NOT_FOUND`"),
    }
}
