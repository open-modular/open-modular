mod module;
mod module_enum;

use proc_macro::TokenStream;

// =================================================================================================
// Engine Macros
// =================================================================================================

#[proc_macro_attribute]
pub fn module(args: TokenStream, input: TokenStream) -> TokenStream {
    module::module_macro(args, input)
}

#[proc_macro_attribute]
pub fn module_enum(args: TokenStream, input: TokenStream) -> TokenStream {
    module_enum::module_enum_macro(args, input)
}
