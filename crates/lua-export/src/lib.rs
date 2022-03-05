extern crate proc_macro;
use core::panic;

use proc_macro::TokenStream;
use syn::__private::quote::quote;
use syn::__private::ToTokens;
use syn::parse_macro_input;
use syn::ItemMod;

// Does nothing, just removes the annotation
#[proc_macro_attribute]
pub fn lua_export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn lua_export_mod(_attr: TokenStream, item: TokenStream) -> TokenStream {
    /*let attributes = parse_macro_input!(attr as AttributeArgs);
    let path = attributes
    .iter()
    .find_map(|attr| {
        let named_value = match attr {
            NestedMeta::Meta(Meta::NameValue(v)) => v,
            _ => return None,
        };
        // Check that the attribute is the "path" attribute
        if !named_value.path.is_ident("path") {
            return None;
        }
        // Get the string literal associated
        match &named_value.lit {
            Lit::Str(s) => Some(s.value()),
            _ => None,
        }
    })
    .unwrap_or_else(|| panic!("lua_export_mod: expected path attribute"));
    */

    let item_cloned = item.clone();
    let mut mod_ast = parse_macro_input!(item_cloned as ItemMod);

    let functions = if let Some((_, items)) = &mod_ast.content {
        items
            .iter()
            .filter_map(|item| {
                let fun = match item {
                    syn::Item::Fn(f) => f,
                    _ => return None,
                };

                // Only export functions with the lua_export attribute
                if fun
                    .attrs
                    .iter()
                    .any(|attr| attr.path.is_ident("lua_export"))
                {
                    Some(LuaDef::from(fun))
                } else {
                    None
                }
            })
            .collect::<Vec<LuaDef>>()
    } else {
        vec![]
    };

    // https://users.rust-lang.org/t/solved-derive-and-proc-macro-add-field-to-an-existing-struct/52307/3
    let lua_code_functions = functions
        .iter()
        .map(|f| f.to_lua_code())
        .collect::<Vec<String>>();

    let function_array = quote! {
        pub const LUA_FUNCTIONS: &[&str] = &[#(#lua_code_functions),*];
    }
    .into();

    let function_item = parse_macro_input!(function_array as syn::Item);

    if let Some((_, items)) = &mut mod_ast.content {
        items.push(function_item);
    } else {
        panic!("lua_export_mod: can't add function array to the module");
    };

    mod_ast.into_token_stream().into()
}

#[derive(Debug)]
struct LuaDef {
    name: String,
    args: Vec<String>,
}

impl From<&syn::ItemFn> for LuaDef {
    fn from(fun: &syn::ItemFn) -> Self {
        let name = fun.sig.ident.to_string();
        let args = fun
            .sig
            .inputs // arguments
            .iter()
            .map(|arg| match arg {
                // Match typed arguments like foo: f64 (not self)
                syn::FnArg::Typed(arg) => match arg.pat.as_ref() {
                    // Match the pattern (only simple identifiers like "foo")
                    syn::Pat::Ident(name) => name.ident.to_string(),
                    _ => panic!("Unsupported argument pattern"),
                },
                _ => panic!("Unsupported argument type"),
            })
            .collect::<Vec<String>>();

        Self { name, args }
    }
}

impl LuaDef {
    fn to_lua_code(&self) -> String {
        let args = self
            .args
            .iter()
            .map(|arg| format!("{}", arg))
            .collect::<Vec<String>>()
            .join(", ");

        format!(
            "function M.{}({})\n\treturn {}({})\nend\n\n",
            self.name, args, self.name, args
        )
    }
}
