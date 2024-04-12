use super::{default, Error};
use std::marker::PhantomData;

/// A pipeline of ETL methods; from input `I` to output `O`.
///
/// ```rust
/// let pipe = Pipe::<I, O>::new();
/// let _ = pipe.map_transform(|| {
///     ...
///     })
///     .await?
///     .load(...)
///     .await?;
pub struct Pipe<I, O> {
    // There's no actual data to hold, so we use PhantomData to remember the 2 types of the process.
    // This way the compiler stays happy, and we have a concise way to declare a new ETL process, i.e.;
    // ```
    // let pipe = Pipe::<I, O>::new();
    // ```
    _phantom: PhantomData<(I, O)>,
}

impl<I, O> Pipe<I, O> 
where
    I: serde::de::DeserializeOwned + Send,
    O: for<'a> serde::de::Deserialize<'a> + serde::Serialize + Send,
{
    /// Initialise an ETL pipeline.
    pub fn new() -> Self {
        Pipe {
            _phantom: PhantomData,
        }
    }

    /// Extract data from some endpoint (e.g., URL or File Path) to a value of input type `I`.
    /// 
    /// - ***path*** --- Path to the endpoint.
    /// 
    /// *The default implementation sends a GET request if `path`starts with `http`, 
    /// and then tries to open a file from `path` if not.*
    pub async fn extract(&self, path: &str) -> Result<I, Error> {
        default::extract(path).await
    }

    /// Transform input type `I` to some output type `O`.
    /// 
    /// - ***input*** --- The transformed data.
    pub async fn transform(&self, _input: I) -> Result<O, Error> {
        unimplemented!() // `required method`: user to re-implement this.
    }

    /// Load output type `O` to some Database.
    /// 
    /// - ***output*** --- The transformed data.
    /// - ***conn*** --- Connection query string for connecting to the database.
    /// - ***doc_id*** --- Name/ID of document/table to update/create within the database.
    pub async fn load(&self, output: O, conn: &str, doc_id: &str) -> Result<(), Error> {
        default::load(output, conn, doc_id).await?;
        Ok(())
    }

    /// [`extract()`] & [`transform()`]
    ///
    /// Extract some value as type `I`, and then transform it to type `O`.
    ///
    /// [`extract()`]: crate::pipe::Pipe::extract
    /// [`transform()`]: crate::pipe::Pipe::transform
    pub async fn extran(&self, path: &str) -> Result<O, Error> {
        let input = self.extract(path).await?;
        self.transform(input).await
    }

    /// [`extract()`] & [`transform()`] & [`load()`]
    ///
    /// Extract some value as type `I`, and then transform it to type `O`, before then loading it to some database.
    ///
    /// [`extract()`]: crate::pipe::Pipe::extract
    /// [`transform()`]: crate::pipe::Pipe::transform
    /// [`load()`]: crate::pipe::Pipe::load
    pub async fn etl(&self, path: &str, conn: &str, doc_id: &str) -> Result<(), Error> {
        let input = self.extract(path).await?;
        let output = self.transform(input).await?;
        self.load(output, conn, doc_id).await
    }

    /// Closure format of [`extract()`].
    /// 
    /// [`extract()`]: crate::pipe::Pipe::extract
    pub async fn map_extract() {}

    /// Closure format of [`transform()`].
    /// 
    /// [`transform()`]: crate::pipe::Pipe::transform
    pub async fn map_transform() {}

    /// Closure format of [`load()`].
    /// 
    /// [`load()`]: crate::pipe::Pipe::load
    pub async fn map_load() {}
}