extern crate proc_macro;
use core::panic;
use std::io::Write;

use proc_macro::TokenStream;
use syn::parse_macro_input;
use syn::AttributeArgs;
use syn::ItemMod;

#[proc_macro_attribute]
pub fn lua_export_mod(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attributes = parse_macro_input!(attr as AttributeArgs);
    let path = attributes
        .iter()
        .find_map(|attr| match attr {
            syn::NestedMeta::Meta(m) => match m {
                syn::Meta::NameValue(v) => match v.path.get_ident() {
                    Some(i) if i == "path" => match v.lit.clone() {
                        syn::Lit::Str(s) => Some(s.value()),
                        _ => panic!("lua_export_mod: path must be a string literal"),
                    },
                    _ => panic!("lua_export_mod: path expected a path"),
                },
                _ => panic!("lua_export_mod: path expected a path"),
            },
            _ => panic!("lua_export_mod: expected a path"),
        })
        .unwrap();

    let item2 = item.clone();
    let mod_input = parse_macro_input!(item2 as ItemMod);

    let functions = if let Some((_, items)) = mod_input.content {
        items
            .iter()
            .map(|item| match item {
                syn::Item::Fn(fun) => {
                    // Checks if there's a #[lua_export] attribute
                    if fun
                        .attrs
                        .iter()
                        .any(|attr| attr.path.is_ident("lua_export"))
                    {
                        Some(fun_to_lua_def(fun))
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .filter_map(|x| x)
            .collect::<Vec<LuaDef>>()
    } else {
        vec![]
    };

    // Open file at path
    let mut file = std::fs::File::create(&path).unwrap();
    // Write the header
    write!(file, "local M = {{}}\n\n").unwrap();
    for f in functions {
        write!(file, "{}", f.to_lua_code()).unwrap();
    }
    write!(file, "return M\n").unwrap();

    // Returns the input
    item
}

fn fun_to_lua_def(fun: &syn::ItemFn) -> LuaDef {
    let name = fun.sig.ident.to_string();
    let args = fun
        .sig
        .inputs
        .iter()
        // Iterate over the arguments
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

    LuaDef { name, args }
}

#[derive(Debug)]
struct LuaDef {
    name: String,
    args: Vec<String>,
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

// Does nothing, just gets the annotation (is this even needed?)
#[proc_macro_attribute]
pub fn lua_export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
