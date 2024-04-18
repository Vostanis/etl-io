use super::{Input, Output};
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
/// ```
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
    I: Input,
    O: Output,
{
    /// Initialise an ETL pipeline.
    pub fn new() -> Self {
        Pipe {
            _phantom: PhantomData,
        }
    }
}
