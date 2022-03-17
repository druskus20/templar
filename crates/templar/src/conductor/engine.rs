use crate::conductor::trebuchet::parser::ParserConfig;
use anyhow::Result;

/*
 * This trait will maybe become a plugin system one day. Will probably need
 * to look into dynamic linking and ABI stuff (Rust doesn't have a stable ABI)
 * woo
*/

pub(crate) trait Engine {
    fn new(config: ParserConfig) -> Self
    where
        Self: Sized;
    fn run(&self, input: &str) -> Result<String>;
}
