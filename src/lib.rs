//! # Extract, Transform, Load
//! 
//! `ETL` is a trait for a serde-Deserialized ***JSON input***, paired with a serde-Serialized ***output***; `ETL<I, O>`.
//! 
//! The 3 core functions (below) are intended to be customized (although, `extract()` and `load()` do have default implementations).
//! 
//! *The aim of this crate is to standardize the typical ETL pipeline.*
//! 
//! The 3 core methods are: 
//!     - `extract(endpoint)` 
//!     - `transform(dataset)`
//!     - `load(transformed_dataset)`
//!
//! By default, `transform()` will always need defining, but `extract()` and `load()` do come with useful default implementations.
//! 
//! ## Aggregrations
//! Subsequent aggregate methods are then derived, for example: 
//!     - `extran(endpoint)` - a combination of `extract()` and `transform()`.
//!     - `etl(endpoint)` - all three processes combined.
//! 
//! ## Example: Definition
//! 
//! ```
//! #[serde(Deserialize)]
//! struct Input {};
//! 
//! #[serde(Serialize)]
//! struct Output {};
//! 
//! impl ETL<Input, Output> for Input {
//!     pub fn transform(data: Input) -> Result<Output, etl::Error> {
//!         ...
//!     }
//! }
//! ```
//! 
//! Default implementations defined where possible, but `transform()` definition is required.
//! 
//! ## Example: Usage
//! ```
//! #[tokio::main]
//! async fn main() {
//!     Input::etl().await;
//! }
//! ```
//! 
//! ### Important footnote on Source Code
//! `async fn` traits are not recommended by the compiler, so when reviewing the source code you may note the follwoing format of function definitions:
//! 
//! ```
//! fn some_function() -> impl Future<Output = Result<T, Error>> + Send { async { ... } }
//! ```
//! 
//! This is contrast to the more typical `async fn` definition:
//! 
//! ```
//! async fn some_function() -> Result<T, Error> { ... }
//! ```
//! 
//! The compiler simply recommends *against* the latter option, suggesting the prior instead.
//! 
//! When declaring your own implementations, simply define in the typical `async fn ...` format.
//! 
//! ## Example
//! ```
//! // Input Type; the struct/enum that deserializes the source material
//! #[derive(Deserialize, Debug)]
//! struct RawPrice {
//!     chart: Chart
//! }
//! 
//! // `Chart` definition omitted for simplicity ... 
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
//!         let data: RawPrice = serde_json::from_str(&init).expect("could not deserialize data to RawPrice type");
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

use std::future::Future;

pub trait ETL<I, O>
where
    I: serde::de::DeserializeOwned + Send, // input needs to Deserialized but required to take ownership
    O: serde::Serialize + Send             // all output can must be Serialized
{
    /// # Extract, Transform, Load
    /// ## Extract
    /// fetch data from some endpoint (A.K.A. path); 
    /// default implementation assumes &str input type resembling either a File Path or a URL
    fn extract(path: &str) -> impl Future<Output = Result<I, Error>> + Send { async move {
        if path.starts_with("http") {
            Self::extract_url(path).await
        } else {
            Self::extract_file(path).await
        }
    }}
    
    /// Read a JSON file and deserialize to some I type
    fn extract_file(file_path: &str) -> impl Future<Output = Result<I, Error>> + Send { async move {
        let file = std::fs::File::open(file_path)?;
        let reader = std::io::BufReader::new(file);
        let data = serde_json::from_reader(reader)?;
        Ok(data)
    }}

    /// GET request a URL (with a client), deserializing the JSON response to some I type
    fn extract_url(url: &str) -> impl Future<Output = Result<I, Error>> + Send { async move {
        let client = reqwest::Client::new();
        let response = client.get(url)
            .header("User-Agent", "Mozilla/5.0")
            .send().await?
            .text().await?;
        let data: I = serde_json::from_str(&response).expect("couldn't deserialize Price");
        Ok(data)
    }}

    /// ## Transform
    /// Transform to desired output (to be written by the user).
    fn transform(input: I) -> impl Future<Output = Result<O, Error>> + Send;

    /// ## Load
    /// Load to a database. Current database APIs are:
    /// - CouchDB
    /// - ScyllaDB
    /// - PostgreSQL
    fn load(output: O, db: Database) -> impl Future<Output = ()> + Send { async move {

        match db {

            // ---> CouchDB
            Database::CouchDB(conn, doc_id) => {
                todo!()
            }

            // ---> ScyllaDB
            Database::ScyllaDB(conn, doc_id) => {
                todo!()
            }

            // ---> PostgreSQL
            Database::PostgreSQL(conn, tbl_name) => {
                todo!()
            }
        }
    }}

    /// # Aggregated Methods
    /// extract() -> transform()
    fn extran(path: &str) -> impl Future<Output = Result<O, Error>> + Send { async {
        let input = Self::extract(path).await.expect("could not extract {self}");
        let output = Self::transform(input).await.expect("could not transform {self}");
        Ok(output)
    }}

    /// extract() -> transform() -> load()
    fn etl(path: &str, db: Database) -> impl Future<Output = ()> + Send { async {
        let input = Self::extract(path).await.expect("could not extract {self}");
        let output = Self::transform(input).await.expect("could not transform {self}");
        Self::load(output, db).await//.expect("could not load {self} to database")
    }}
}

/// enum used in specifying which Database API to use.
/// Allows for a generalised `load()` method to be maintained.
pub enum Database<'a> {
    CouchDB(&'a str, &'a str),
    ScyllaDB(&'a str, &'a str),
    PostgreSQL(&'a str, &'a str)
}

/// Custom Error type for ETL, wrapping all Error types.
/// If a user adds further dependencies, they should redefine further objects within Error enum.
/// If not, an undefined error will return an 'anyhow::Error' wrapping (defined as Other).
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("[error] could not fetch URL: {0}")]
    HTTP(#[from] reqwest::Error),

    #[error("[error] could not open file: {0}")]
    IO(#[from] std::io::Error),

    #[error("[error] could not convert source to JSON: {0}")]
    JSON(#[from] serde_json::Error),

    #[error("[error] {0}")]
    Other(#[from] anyhow::Error)
}