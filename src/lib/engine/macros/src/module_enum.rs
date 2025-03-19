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
use syn::ItemEnum;
use uuid::Uuid;

// =================================================================================================
// Module Enumeration
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

pub(crate) fn module_enum_macro(args: TokenStream, input: TokenStream) -> TokenStream {
    Args::parse(args).map_or_else(
        |e| e,
        |a| module_enum_tokens(&a, &syn::parse_macro_input!(input as ItemEnum)).into(),
    )
}

// Tokens

#[allow(clippy::too_many_lines)]
pub(crate) fn module_enum_tokens(args: &Args, item: &ItemEnum) -> TokenStream2 {
    // Item

    let attrs = &item.attrs;
    let ident = &item.ident;
    let generics = &item.generics;
    let vis = &item.vis;
    let where_clause = &item.generics.where_clause;

    // Derived

    let type_params: Vec<_> = generics.type_params().cloned().collect();

    assert_eq!(type_params.len(), 1, "types != 1");

    let type_param = &type_params[0];

    // Variants

    let variant = item
        .variants
        .iter()
        .map(|v| v.ident.clone())
        .collect::<Vec<_>>();

    // Id

    let uuid = Uuid::from_str(&args.id).expect("valid id");
    let uuid_value = uuid.as_bytes();

    // Module

    let module = format_ident!("{}_generated", ident.to_string().to_case(Case::Snake));

    // Tokens

    quote::quote! {
        #(#attrs)*
        #vis enum #ident #generics #where_clause {
            #(#variant(#variant #generics)),*
        }

        mod #module {
            use super::*;

            // AsMut<Connector<'rt>>

            impl #generics AsMut<::open_modular_engine::node::Node> for #ident #generics #where_clause {
                fn as_mut(&mut self) -> &mut ::open_modular_engine::node::Node {
                    match self {
                        #(Self::#variant(module) => module.as_mut()),*
                    }
                }
            }

            // AsRef<Connector<'rt>>

            impl #generics AsRef<::open_modular_engine::node::Node> for #ident #generics #where_clause {
                fn as_ref(&self) -> & ::open_modular_engine::node::Node {
                    match self {
                        #(Self::#variant(module) => module.as_ref()),*
                    }
                }
            }

            // From<ModuleA<'rt>>, From<ModuleB<'rt>>, etc.

        #(
            impl #generics From<#variant #generics> for #ident #generics #where_clause {
                fn from(module: #variant #generics) -> Self {
                    Self::#variant(module)
                }
            }
        )*

            // Identify

            impl #generics ::open_modular_engine::module::Identify for #ident #generics #where_clause {
                fn id() -> ::open_modular_engine::_dependencies::uuid::Uuid {
                    Self::id().clone()
                }
            }

            // Module

            impl #generics ::open_modular_engine::module::Module for #ident #generics #where_clause {}

            // Module Source

            impl #generics::open_modular_engine::module::ModuleSource for #ident #generics #where_clause {
                type Context = R;

                fn instantiate(
                    id: &::open_modular_engine::_dependencies::uuid::Uuid,
                    context: Self::Context
                ) -> Self {
                    let definitions = Self::definitions();
                    let definition = definitions.get(id).unwrap();
                    let instantiations = Self::instantiations();
                    let instantiation = instantiations.get(id).unwrap();
                    let node = ::open_modular_engine::node::Node::from_definition(definition);

                    instantiation(node, context)
                }
            }

            // Process

            impl #generics ::open_modular_engine::module::Process for #ident #generics #where_clause {
                fn process(&mut self, args: &::open_modular_engine::module::ProcessArgs) {
                    match self {
                        #(Self::#variant(module) => module.process(args)),*
                    }
                }
            }

            // Constant Functions

            impl #generics #ident #generics #where_clause {
                #[doc(hidden)]
                const fn definitions() ->
                    ::std::sync::LazyLock<
                        ::std::collections::HashMap<
                            ::open_modular_engine::_dependencies::uuid::Uuid,
                            ::open_modular_engine::module::ModuleDefinition
                        >
                    >
                {
                    ::std::sync::LazyLock::new(|| {
                        let mut definitions = ::std::collections::HashMap::new();

                    #(
                        definitions.insert(
                            <#variant::#generics as ::open_modular_engine::module::Identify>::id(),
                            <#variant::#generics as ::open_modular_engine::module::Define>::define(
                                ::open_modular_engine::module::ModuleDefinition::builder()
                            ).into(),
                        );
                    )*

                        definitions
                    })
                }

                #[doc(hidden)]
                const fn instantiations() ->
                    ::std::sync::LazyLock<
                        ::std::collections::HashMap<
                            ::open_modular_engine::_dependencies::uuid::Uuid,
                            Box<dyn Fn(
                                ::open_modular_engine::node::Node,
                                #type_param,
                            ) -> #ident #generics>
                        >
                    >
                {
                    ::std::sync::LazyLock::new(|| {
                        let mut instantiations = ::std::collections::HashMap::new();

                    #(
                        instantiations.insert(
                            <#variant::#generics as ::open_modular_engine::module::Identify>::id(),
                            Box::new(|node, context| {
                                #ident::#variant(
                                    <#variant::#generics as ::open_modular_engine::module::Instantiate>::instantiate(
                                        node,
                                        context,
                                    )
                                )
                            })
                            as Box<dyn Fn(
                                ::open_modular_engine::node::Node,
                                #type_param,
                            ) -> #ident #generics>
                        );
                    )*

                        instantiations
                    })
                }

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
