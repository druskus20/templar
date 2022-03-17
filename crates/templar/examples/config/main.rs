use anyhow::Result;
use rlua::prelude::*;
use std::path::PathBuf;

fn main() -> Result<()> {
    // TODO: Fix relative path when executed from the project root
    std::env::set_current_dir("./examples/config")?;

    let lua = Lua::new();
    templar::config::api::register_lua_api(&lua)?;
    templar::config::api::gen_lua_wrapper(PathBuf::from("example.lua"))?;

    templar::config::require_config(&lua, PathBuf::from("./config.lua"))?;
    Ok(())
}
