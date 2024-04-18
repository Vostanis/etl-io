use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{braced, parse_macro_input, Attribute, Block, Stmt, Token, Type};

/// # Pipeline!
/// For each Input/Output pair that you declare, you will need `exchange()`, `transform()` and `load()`.
/// 
/// To standardize the process, we can use the `pipeline!` macro like so:
/// ```rust
/// pipeline! {
/// //  ._________________.__________ the macro utilises custom syntax of the form: `@ SomeInputType -> SomeOutputType { ... }`
/// //  |                 |               
///     @ OriginalFormat -> OutputFormat
///     {
///         // using the default impl of extract()
/// 
///         async fn transform(&self, input: OriginalFormat) -> Result<OutputFormat, pipe_io::Error> { 
///             // ... 
///         }
/// 
///         // using the default impl of laod()
///     }
/// 
///     @ AnotherOriginal -> AnotherOutput
///     {
///         // using the default impl of extract()
/// 
///         async fn transform(&self, input: AnotherOriginal) -> Result<AnotherOutput, pipe_io::Error> { 
///             // ... 
///         }
/// 
///         async fn load(self, output: AnotherOutput, conn: &str, doc_id: &str) -> Result<(), pipe_io::Error> { .
///             // ... 
///         }
///     }
/// 
///     @ SomeInput -> AThirdOutput // <------ each Input/Output pair has its own declaration; `@ ... -> ... { ... }`
///     {
///         async fn extract(&self, path: &str) -> Result<SomeInput, pipe_io::Error> {
///             // ... 
///         }
/// 
///         async fn transform(&self, input: SomeInput) -> Result<AThirdOutput, pipe_io::Error> {
///             // ...
///         }
/// 
///         async fn load(&self, output: SomeInput, conn: &str, doc_id: &str) -> Result<(), pipe_io::Error> {
///             // ...
///         }
///     }
/// }
/// ```
/// # Note
/// In each case, `transform()` had to be defined - this is because there's no common-sense default implementation that we can write for all cases.
#[proc_macro]
pub fn pipeline(input: TokenStream) -> TokenStream {

    let block = parse_macro_input!(input as Args);

    // push all Pipe statements to a vector
    let mut quotes: Vec<proc_macro2::TokenStream> = vec![];
    for arg in block.args {
        let input_type = &arg.type_one; // match Type::Path (MyStruct) or Type::Group (Vec<MyStruct>)
        let output_type = &arg.type_two; // match Type::Path (MyStruct) or Type::Group (Vec<MyStruct>)
        let stmts = &arg.stmts;
        quotes.push(quote! { // <--- collect all Pipe statements
            impl pipe_io::Input for #input_type {}
            impl pipe_io::Output for #output_type {}
            impl pipe_io::ETL<#input_type, #output_type> for pipe_io::Pipe<#input_type, #output_type>
            {
                #(#stmts)*
            }
        })
    }

    quote! { #(#quotes)* }.into() // <--- rewrite out all the Pipe statements
}

// a single `pipe` input; `@ SomeInput -> SomeOutput { ... }`
struct Arg {
    type_one: Type,
    type_two: Type,
    stmts: Vec<Stmt>,
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> Result<Self> {
        // `@ Input -> Output`
        input.parse::<Token![@]>()?;
        let type_one = input.parse()?;
        input.parse::<Token![->]>()?;
        let type_two = input.parse()?;

        // `{ async fn func() { ... } ... }`
        let brace_content;
        let _brace_token = braced!(brace_content in input);
        let _inner_brace_attrs = brace_content.call(Attribute::parse_inner)?;
        let stmts = brace_content.call(Block::parse_within)?;
        Ok(Arg {
            type_one,
            type_two,
            stmts,
        })
    }
}

// collect all the `pipe`s together in a vector
struct Args {
    args: Vec<Arg>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut args = vec![];
        while !input.is_empty() {
            args.push(input.parse()?);
        }
        Ok(Args { args })
    }
}
