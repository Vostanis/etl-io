use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{braced, bracketed, parse_macro_input, Attribute, Block, Ident, Stmt, Token, Type, TypePath};

#[proc_macro]
pub fn etl(input: TokenStream) -> TokenStream {
    let arg = parse_macro_input!(input as Arg);

    let type1 = &arg.type_one;
    let type2 = &arg.type_two;
    let stmts = &arg.stmts;

    quote! {
        impl pipe_io::Input for #type1 {}
        impl pipe_io::Output for #type2 {}
        impl pipe_io::ETL<#type1, #type2> for pipe_io::Pipe<#type1, #type2>
        {
            #(#stmts)*
        }
    }
    .into()
}

struct Arg {
    type_one: Ident,
    type_two: Ident,
    stmts: Vec<Stmt>,
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> Result<Self> {

        // @ Input -> Output
        input.parse::<Token![@]>()?;
        let type_one: Ident = input.parse()?;
        input.parse::<Token![->]>()?;
        let type_two: Ident = input.parse()?;

        // @[Input -> Output]
        // let bracket_content;
        // let _bracket_token = bracketed!(bracket_content in input);
        // let type_one: Type = bracket_content.parse()?;
        // bracket_content.parse::<Token![-]>()?;
        // bracket_content.parse::<Token![>]>()?;
        // let type_two: Type = bracket_content.parse()?;

        // { async fn func() { ... } ... }
        let brace_content;
        let _brace_token = braced!(brace_content in input);
        let _inner_brace_attrs = brace_content.call(Attribute::parse_inner)?;
        let stmts = brace_content.call(Block::parse_within)?;

        // return 1 Arg - can this be abstracted to a Vec<Arg>?
        Ok(Arg {
            type_one,
            type_two,
            stmts,
        })
    }
}