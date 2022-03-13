use anyhow::Result;

/*
 * This trait will maybe become a plugin system one day. Will probably need
 * to look into dynamic linking and ABI stuff (Rust doesn't have a stable ABI)
 * woo
*/

pub(crate) trait Engine {
    fn run(&self, input: &str) -> Result<String>;
}
