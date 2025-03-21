use std::str::FromStr;

use convert_case::{
    Case,
    Casing,
};
use darling::{
    Error,
    FromMeta,
    ast::NestedMeta,
};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::format_ident;
use syn::ItemStruct;
use uuid::Uuid;

// =================================================================================================
// Module
// =================================================================================================

// Args

#[derive(Debug, FromMeta)]
pub(crate) struct Args {
    id: String,
}

impl Args {
    pub fn parse(args: TokenStream) -> Result<Self, TokenStream> {
        NestedMeta::parse_meta_list(args.into())
            .map_err(|e| Error::from(e).write_errors().into())
            .and_then(|a| Self::from_list(&a).map_err(|e| e.write_errors().into()))
    }
}

// Macro

pub(crate) fn module_macro(args: TokenStream, input: TokenStream) -> TokenStream {
    Args::parse(args).map_or_else(
        |e| e,
        |a| module_tokens(&a, &syn::parse_macro_input!(input as ItemStruct)).into(),
    )
}

// Tokens

pub(crate) fn module_tokens(args: &Args, item: &ItemStruct) -> TokenStream2 {
    // Item

    let ident = &item.ident;
    let generics = &item.generics;
    let where_clause = &item.generics.where_clause;

    // Module

    let module = format_ident!("{}_generated", ident.to_string().to_case(Case::Snake));

    // Id

    let uuid = Uuid::from_str(&args.id).expect("valid id");
    let uuid_value = uuid.as_bytes();

    // Tokens

    quote::quote! {
        #item

        mod #module {
            use super::*;

            // AsMut<PortOutputs>

            impl #generics AsMut<::open_modular_engine::port::PortOutputs> for #ident #generics #where_clause {
                fn as_mut(&mut self) -> &mut ::open_modular_engine::port::PortOutputs {
                    &mut self.port_outputs
                }
            }

            // AsRef<PortInputs>

            impl #generics AsRef<::open_modular_engine::port::PortInputs> for #ident #generics #where_clause {
                fn as_ref(&self) -> & ::open_modular_engine::port::PortInputs {
                    &self.port_inputs
                }
            }

            // Identify

            impl #generics ::open_modular_engine::module::Identify for #ident #generics #where_clause {
                fn id() -> ::open_modular_engine::_dependencies::uuid::Uuid {
                    Self::id().clone()
                }
            }

            // Module

            impl #generics ::open_modular_engine::module::Module for #ident #generics #where_clause {}

            // Constant Functions

            impl #generics #ident #generics #where_clause {
                #[doc(hidden)]
                const fn id() -> ::std::sync::LazyLock<::open_modular_engine::_dependencies::uuid::Uuid> {
                    ::std::sync::LazyLock::new(|| {
                        ::open_modular_engine::_dependencies::uuid::Uuid::from_bytes([
                            #(#uuid_value),*
                        ])
                    })
                }
            }
        }
    }
}
