use crate::conductor::trebuchet::parser::ParserConfig;
use anyhow::Result;
use dyn_clone::DynClone;

/*
 * This trait will maybe become a plugin system one day. Will probably need
 * to look into dynamic linking and ABI stuff (Rust doesn't have a stable ABI)
 * woo
*/

dyn_clone::clone_trait_object!(Engine);

pub(crate) trait Engine: DynClone {
    fn new(config: ParserConfig) -> Self
    where
        Self: Sized;
    fn run(&self, input: &str) -> Result<String>;
}
