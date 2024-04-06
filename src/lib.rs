//! # Extract, Transform, Load
//!
//! `ETL` is a trait for a serde-Deserialized ***JSON input***, paired with a serde-Serialized ***output***; `ETL<I, O>`.
//! 
//! *The aim of this crate is to standardize the typical ETL pipeline for JSON.*
//!
//! The 3 core functions (below) are intended to be customized (although, `extract()` and `load()` do have default implementations).
//!
//! The 3 core methods are:
//! - `extract(endpoint)`
//! - `transform(dataset)`
//! - `load(transformed_dataset)`
//!
//! By default, `transform()` will always need defining, but `extract()` and `load()` do come with useful default implementations.
//!
//! ## Aggregrations
//! Subsequent aggregate methods are then derived, for example:
//! - `extran(endpoint)` - a combination of `extract()` and `transform()`.
//! - `etl(endpoint)` - all three processes combined.
//!
//! ## Example 1: Generic
//!
//! ```
//! #[serde(Deserialize)]
//! struct Input {
//!     ...
//! };
//!
//! #[serde(Serialize)]
//! struct Output {
//!     ...
//! };
//!
//! impl ETL<Input, Output> for Input {
//! 
//!     // using extract() default impl
//! 
//!     pub fn transform(data: Input) -> Result<Output, etl::Error> {
//!         ...
//!     }
//! 
//!     // using load() default impl
//! } 
//!
//! #[tokio::main]
//! async fn main() {
//!     Input::etl().await;
//! }
//! ```
//!
//! Default implementations defined where possible, but a `transform()` definition is always required by the user.
//! 
//! ## Example 2: Specific
//! ```
//! // Input Type; the struct/enum that deserializes the source material
//! #[derive(Deserialize, Debug)]
//! struct RawPrice {
//!     chart: Chart
//! }
//! // The rest of the `Chart` definition has been omitted for simplicity ...
//!
//! // Output Type; the struct/enum that serializes to the final output
//! #[derive(Serialize, Deserialize, Debug)]
//! struct Price {
//!     date: String,
//!     open: f64,
//!     high: f64,
//!     low: f64,
//!     close: f64,
//!     adj_close: f64,
//!     volume: u64,
//! }
//!
//! // The following is the impl, which could be written one of two ways (in generic terms):
//! //      impl ETL<I, O> for I { ... }
//! //      impl ETL<I, O> for O { ... }
//! impl ETL<RawPrice, Vec<Price>> for RawPrice
//! {
//!     // reading JSON from static str
//!     async fn extract(init: &str) -> Result<RawPrice, etl::Error> {
//!         let data: RawPrice = serde_json::from_str(&init)?;
//!         Ok(data)
//!     }
//!
//!     // method that unpacks Input into Output (the following steps are situational)
//!     async fn transform(data: RawPrice) -> Result<Vec<Price>, etl::Error> {
//!         let base = &data.chart.result[0];
//!         let price = &base.indicators.quote[0];
//!         let adjclose = &base.indicators.adjclose[0].adjclose;
//!         let dates = &base.date;
//!         let price_set = price
//!             .open.iter()
//!             .zip(price.high.iter())
//!             .zip(price.low.iter())
//!             .zip(price.close.iter())
//!             .zip(price.volume.iter())
//!             .zip(adjclose.iter())
//!             .zip(dates.iter())
//!             .map(|((((((open, high), low), close), volume), adj_close), date)| Price {
//!                 date: date.clone(),
//!                 open: *open,
//!                 high: *high,
//!                 low: *low,
//!                 close: *close,
//!                 adj_close: *adj_close,
//!                 volume: *volume,
//!             })
//!             .collect::<Vec<_>>();
//!         Ok(price_set)
//!     }
//! }
//! ```
//!
//! ### Important footnote on Source Code
//! `async traits` are not recommended by the compiler, so when reviewing the source code you may note the following format of function definitions:
//!
//! ```
//! fn some_function() -> impl Future<Output = Result<T, Error>> { async { ... } }
//! ```
//!
//! This is in contrast to the typical `async fn` definition:
//! 
//! ```
//! async fn some_function() -> Result<T, Error> { ... }
//! ```
//!
//! The compiler simply recommends *against* the latter option, suggesting the prior instead, when building `async traits`.
//!
//! When declaring your own implementations, define them in the typical `async fn ...` format.

pub mod db;
use crate::db::couchdb;

use std::future::Future;
use std::marker::PhantomData;

pub struct Node<I, O>
where
    I: serde::de::DeserializeOwned + Send,
    O: for<'a> serde::de::Deserialize<'a> + serde::Serialize + Send
{
    /// We don't want to store an object of type I or of type O (or both). 
    /// 
    /// However, if we don't have *some object* (something to refer to as *self*), then we cannot safely
    /// apply any derived methods (see [Object Safety](https://doc.rust-lang.org/reference/items/traits.html#object-safety)).
    /// 
    /// So, to keep the compiler happy, we can use PhantomData<T>.
    _phantom: PhantomData<(I, O)>
}

impl<I, O> Node<I, O> 
where
    I: serde::de::DeserializeOwned + Send,
    O: for<'a> serde::de::Deserialize<'a> + serde::Serialize + Send
{
    /// Creates an ETL node
    pub async fn create_node() -> Node<I, O> {
        Node { 
            _phantom: PhantomData
        }
    }
}

impl<I, O> ETL<I, O> for Node<I, O> 
where
    I: serde::de::DeserializeOwned + Send,
    O: for<'a> serde::de::Deserialize<'a> + serde::Serialize + Send 
{
    async fn transform(&self, _input: I) -> Result<O, Error> {
        unimplemented!()
    }
}

pub trait ETL<I, O>
where
    I: serde::de::DeserializeOwned + Send,
    O: for<'a> serde::de::Deserialize<'a> + serde::Serialize + Send,
{
    /// Fetch data from some endpoint (A.K.A. path);
    /// default implementation assumes `&str` input type, resembling either a File Path or a URL.
    fn extract(&self, path: &str) -> impl Future<Output = Result<I, Error>> {
        async move {
            if path.starts_with("http") {
                self.extract_url(path).await
            } else {
                self.extract_file(path).await
            }
        }
    }

    /// Reads a JSON file and deserializes to some `I` type.
    fn extract_file(&self, file_path: &str) -> impl Future<Output = Result<I, Error>> {
        async move {
            let file = std::fs::File::open(file_path)?;
            let reader = std::io::BufReader::new(file);
            let data = serde_json::from_reader(reader)?;
            Ok(data)
        }
    }

    /// GET request a URL (with a client), deserializing the JSON response to some `I` type.
    /// 
    /// Provides 1 (very cheeky & anonymous) HTTP header: 
    /// 
    /// `{ "User-Agent":"example@example.com" }`
    fn extract_url(&self, url: &str) -> impl Future<Output = Result<I, Error>> {
        async move {
            let client = reqwest::Client::new();
            let response = client
                .get(url)
                .header("User-Agent", "example@example.com")
                .send()
                .await?
                .text()
                .await?;
            let data: I = serde_json::from_str(&response).expect("couldn't deserialize Price");
            Ok(data)
        }
    }

    /// Transform to desired output (to be written by the user).
    fn transform(&self, input: I) -> impl Future<Output = Result<O, Error>>;

    /// Load to a database.
    /// The default implementation has a list of current database APIs:
    /// - CouchDB
    // - ScyllaDB
    // - PostgreSQL
    ///
    /// The assumed workflow is: take a file/table and create/update it
    fn load(&self, output: O, conn: &str, doc_id: &str) -> impl Future<Output = Result<(), Error>> {
        async move {
            let _ = self.load_couchdb(output, conn, doc_id).await;
            Ok(())
        }
    }

    /// Loads document to CouchDB.
    fn load_couchdb(&self, output: O, conn: &str, doc_id: &str) -> impl Future<Output = Result<(), Error>> { 
        async move {
            couchdb::insert_doc::<O>(&output, conn, doc_id).await;
            Ok(())
        }
    }

    /// ## Aggregations

    /// extract() -> transform()
    fn extran(&self, path: &str) -> impl Future<Output = Result<O, Error>> {
        async {
            let input = self.extract(path).await.expect("could not extract {self}");
            let output = self.transform(input)
                .await
                .expect("could not transform {self}");
            Ok(output)
        }
    }

    /// extract() -> transform() -> load()
    fn etl(&self, path: &str, conn: &str, doc_id: &str) -> impl Future<Output = Result<(), Error>> {
        async {
            let input = self.extract(path).await.expect("could not extract {self}");
            let output = self.transform(input)
                .await
                .expect("could not transform {self}");
            self.load(output, conn, doc_id).await //.expect("could not load {self} to database")
        }
    }
}

/// Custom Error wrapper for ETL.
/// 
/// If a user adds further dependencies, they should redefine further objects within Error enum.
/// 
/// If not: any undefined error will return an `anyhow::Error`, defined as `Other(Error)`.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// reqwest
    #[error("[error] could not fetch URL: {0}")]
    HTTP(#[from] reqwest::Error),

    /// std::fs
    #[error("[error] could not open file: {0}")]
    IO(#[from] std::io::Error),

    /// serde_json
    #[error("[error] could not convert source to JSON: {0}")]
    JSON(#[from] serde_json::Error),

    /// undefined errors are umbrella'd under here
    #[error("[error] {0}")]
    Other(#[from] anyhow::Error),
}