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
//! ```rust
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
//! ```rust
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

// Modules
pub mod db;
pub mod default;
pub mod error;
pub mod etl;
pub mod pipe;

// Re-exports
pub use error::Error;
pub use etl::ETL;
pub use macros::pipeline;
pub use pipe::Pipe;

// Crate-wide traits
pub trait Input: serde::de::DeserializeOwned + Send {}
pub trait Output: serde::de::DeserializeOwned + serde::Serialize + Send {}

// Prelude: Commonly Packaged
pub mod prelude {
    pub use super::{Error, Input, Output, Pipe, ETL};
}