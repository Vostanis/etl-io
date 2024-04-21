use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{braced, parse_macro_input, Attribute, Block, Stmt, Token, Type};

////////////////////////////////////////////////////////////////////////////////////////////////////////////
// pipeline! { ... }
////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[proc_macro]
pub fn pipeline(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as Args);
    let mut quotes = vec![];

    // for each Pipe defined, collect each TokenStream into a vector
    for arg in args.args {
        let type1 = &arg.type_one; // match Type::Path (MyStruct) or Type::Group (Vec<MyStruct>)
        let type2 = &arg.type_two; // match Type::Path (MyStruct) or Type::Group (Vec<MyStruct>)
        let stmts = &arg.stmts;
        quotes.push(quote! {
            impl pipe_io::Input for #type1 {}
            impl pipe_io::Output for #type2 {}
            impl pipe_io::ETL<#type1, #type2> for pipe_io::Pipe<#type1, #type2>
            {
                #(#stmts)*
            }
        })
    }

    // print each statement from `quotes`, the vector of TokenStreams
    quote! { #(#quotes)* }.into()
}

// a single `pipe` input
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

////////////////////////////////////////////////////////////////////////////////////////////////////////////
// pipe!(ThisStructOrEnum, ThatStructOrEnum)
////////////////////////////////////////////////////////////////////////////////////////////////////////////

// pipe!(MyStruct, AnotherStruct) == `Pipe::<MyStruct, AnotherStruct>::new()`
#[proc_macro]
pub fn pipe(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as PipeArg);
    let type1 = &args.type_one;
    let type2 = &args.type_two;
    quote! { pipe_io::Pipe::<#type1, #type2>::new() }.into()
}

struct PipeArg {
    type_one: Type,
    type_two: Type,
}

impl Parse for PipeArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let types = Punctuated::<_, Token![,]>::parse_terminated(input)?;
        if types.len() != 2 {
            return Err(input.error("Expected two types separated by a comma"));
        }

        let mut types = types.into_iter();
        let type_one = types.next().unwrap();
        let type_two = types.next().unwrap();

        Ok(PipeArg {
            type_one,
            type_two,
        })
    }
}