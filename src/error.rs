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
