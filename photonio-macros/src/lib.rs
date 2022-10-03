//! Procedural macros for PhotonIO.

#![deny(unused_must_use)]
#![warn(missing_docs, unreachable_pub)]
#![allow(clippy::new_without_default)]

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;

/// Marks a function to be run on a runtime.
///
/// # Examples
///
/// ```no_run
/// use photonio::fs::File;
///
/// #[photonio::main(num_threads = 4)]
/// async fn main() -> std::io::Result<()> {
///     let mut file = File::create("hello.txt").await?;
///     file.write_all(b"hello").await?;
/// }
/// ```
///
/// This is equivalent to:
///
/// ```no_run
/// use photonio::{fs::File, runtime::Builder};
///
/// fn main() -> std::io::Result<()> {
///     let rt = Builder::new().num_threads(4).build()?;
///     rt.block_on(async {
///         let mut file = File::create("hello.txt").await?;
///         file.write_all(b"hello").await?;
///     })
/// }
/// ```
#[proc_macro_attribute]
pub fn main(attr: TokenStream, item: TokenStream) -> TokenStream {
    transform(attr, item, false)
}

/// This is similar to [`macro@main`], but for tests.
#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
    transform(attr, item, true)
}

fn transform(attr: TokenStream, item: TokenStream, is_test: bool) -> TokenStream {
    let opts = match Options::parse(attr.clone()) {
        Ok(opts) => opts,
        Err(e) => return token_stream_with_error(attr, e),
    };
    let mut func: syn::ItemFn = match syn::parse(item.clone()) {
        Ok(func) => func,
        Err(e) => return token_stream_with_error(item, e),
    };

    let head = if opts.env_logger {
        quote! { let _ = env_logger::builder().is_test(true).try_init(); }
    } else {
        quote! {}
    };

    let mut rt = quote! {
        photonio::runtime::Builder::new()
    };
    if let Some(v) = opts.num_threads {
        rt = quote! { #rt.num_threads(#v) }
    }

    func.sig.asyncness = None;
    let block = func.block;
    func.block = syn::parse2(quote! {
        {
            #head;
            let block = async #block;
            #rt.build().expect("failed to build runtime").block_on(block)
        }
    })
    .unwrap();

    let test = if is_test {
        quote! { #[::std::prelude::v1::test] }
    } else {
        quote! {}
    };

    quote! {
        #test
        #func
    }
    .into()
}

#[derive(Default)]
struct Options {
    env_logger: bool,
    num_threads: Option<usize>,
}

type Attributes = syn::punctuated::Punctuated<syn::MetaNameValue, syn::Token![,]>;

impl Options {
    fn parse(input: TokenStream) -> Result<Self, syn::Error> {
        let mut opts = Options::default();
        let attrs = Attributes::parse_terminated.parse(input)?;
        for attr in attrs {
            let name = attr
                .path
                .get_ident()
                .ok_or_else(|| syn::Error::new_spanned(&attr, "missing attribute name"))?
                .to_string();
            match name.as_str() {
                "env_logger" => {
                    opts.env_logger = true;
                }
                "num_threads" => {
                    opts.num_threads = Some(parse_int(&attr.lit)?);
                }
                _ => return Err(syn::Error::new_spanned(&attr, "unknown attribute name")),
            }
        }
        Ok(opts)
    }
}

fn parse_int(lit: &syn::Lit) -> Result<usize, syn::Error> {
    if let syn::Lit::Int(i) = lit {
        if let Ok(v) = i.base10_parse() {
            return Ok(v);
        }
    }
    Err(syn::Error::new(lit.span(), "failed to parse int"))
}

fn token_stream_with_error(mut item: TokenStream, error: syn::Error) -> TokenStream {
    item.extend(TokenStream::from(error.into_compile_error()));
    item
}
