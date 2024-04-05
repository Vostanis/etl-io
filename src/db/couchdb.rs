use serde::{Serialize, Deserialize};
use serde_json::json;

/// # Inserting a new/existing document
///   initial GET --------- fail ------------------ else -------|
/// 
///     |                               |                       |
///   success (file exists)     PUT doc with no id             error
/// 
///     |
/// 
///   PUT doc with id

#[derive(Deserialize, Serialize, Debug, Clone)]
struct CouchDocument {
    _id: String,
    _rev: String,
}

pub async fn insert_doc<T>(url: &str, doc_id: &str, data: &T)
where
    for<'de> T: Serialize + Deserialize<'de>
{
    // check if the document already exists with a GET request
    let url = format!("{url}/{doc_id}");
    let client = reqwest::Client::new();
    let response = client.get(url.clone()).send().await.expect("failed to retrieve GET response");
    let status = response.status();

    match status {

        StatusCode::OK => { // "if the file already exists ..."

            // retrieve current revision id
            let text = response.text().await.expect("failed to translate response to text");
            let current_file: CouchDocument = serde_json::from_str(&text).expect("failed to read current revision to serde schema");

            // PUT file up with current revision id 
            let mut updated_file = json!(data);
            updated_file["_rev"] = json!(current_file._rev);
            // println!("{:#?} : {:#?}", &updated_file["_id"], &updated_file["_rev"]);
            let _second_response = client
                .put(url)
                .json(&updated_file)
                .send()
                .await
                .expect("failed to retrieve PUT response");
        },

        StatusCode::NOT_FOUND => { // "if the file does not exist ..."

            // PUT new file up (requiring no revision id)
            let new_file = json!(data);
            let _second_response = client
                .put(url)
                .json(&new_file)
                .send()
                .await
                .expect("failed to retrieve PUT response");
        },

        _ => println!("the unexpected happened"),
    }
}