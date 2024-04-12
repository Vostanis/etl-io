use super::{default, Error, Input, Output};
use std::future::Future;

pub trait ETL<I, O>
where
    I: Input,
    O: Output,
{
    /// Extract data from some endpoint (e.g., URL or File Path) to a value of input type `I`.
    ///
    /// - ***path*** --- Path to the endpoint.
    ///
    /// *The default implementation sends a GET request if `path`starts with `http`,
    /// and then tries to open a file from `path` if not.*
    fn extract(&self, path: &str) -> impl Future<Output = Result<I, Error>> {
        async { default::extract(path).await }
    }

    /// Transform input type `I` to some output type `O`.
    ///
    /// - ***input*** --- The transformed data.
    fn transform(&self, _input: I) -> impl Future<Output = Result<O, Error>>;

    /// Load output type `O` to some Database.
    ///
    /// - ***output*** --- The transformed data.
    /// - ***conn*** --- Connection query string for connecting to the database.
    /// - ***doc_id*** --- Name/ID of document/table to update/create within the database.
    fn load(&self, output: O, conn: &str, doc_id: &str) -> impl Future<Output = Result<(), Error>> {
        async { default::load(output, conn, doc_id).await }
    }

    /// [`extract()`] & [`transform()`]
    ///
    /// Extract some value as type `I`, and then transform it to type `O`.
    ///
    /// [`extract()`]: crate::pipe::Pipe::extract
    /// [`transform()`]: crate::pipe::Pipe::transform
    fn extran(&self, path: &str) -> impl Future<Output = Result<O, Error>> {
        async {
            let input = self.extract(path).await?;
            self.transform(input).await
        }
    }

    /// [`extract()`] & [`transform()`] & [`load()`]
    ///
    /// Extract some value as type `I`, and then transform it to type `O`, before then loading it to some database.
    ///
    /// [`extract()`]: crate::pipe::Pipe::extract
    /// [`transform()`]: crate::pipe::Pipe::transform
    /// [`load()`]: crate::pipe::Pipe::load
    fn etl(&self, path: &str, conn: &str, doc_id: &str) -> impl Future<Output = Result<(), Error>> {
        async {
            let input = self.extract(path).await?;
            let output = self.transform(input).await?;
            self.load(output, conn, doc_id).await
        }
    }

    /// Closure format of [`extract()`].
    ///
    /// [`extract()`]: crate::pipe::Pipe::extract
    fn map_extract() {
        unimplemented!()
    }

    /// Closure format of [`transform()`].
    ///
    /// [`transform()`]: crate::pipe::Pipe::transform
    fn map_transform() {
        unimplemented!()
    }

    /// Closure format of [`load()`].
    ///
    /// [`load()`]: crate::pipe::Pipe::load
    fn map_load() {
        unimplemented!()
    }
}
