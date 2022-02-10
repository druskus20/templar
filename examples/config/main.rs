use anyhow::Result;
use rlua::prelude::*;
use std::path::PathBuf;

fn main() -> Result<()> {
    std::env::set_current_dir("./examples/config")?;

    let lua = Lua::new();
    templar::config::api::register_lua_api(&lua)?;
    templar::config::load_config(&lua, PathBuf::from("./config.lua"))?;
    Ok(())
}
