use std::path::PathBuf;

use anyhow::Result;
use rlua::Lua;

use super::opt::{Generate, Run};

pub(super) fn run(run: &Run) -> Result<()> {
    let lua = Lua::new();
    super::config::api::register_lua_api(&lua)?;

    let config_path = if let Some(path) = run.config_path.as_ref() {
        PathBuf::from(path).canonicalize()?
    } else {
        let base = match std::env::var("TEMPLAR_CONFIG") {
            Ok(path) => PathBuf::from(path),
            Err(_) => {
                let home = std::env::var("HOME")?;
                PathBuf::from(
                    std::env::var("XDG_CONFIG_HOME")
                        .unwrap_or_else(|_| format!("{}/.config/templar", home)),
                )
            }
        };
        base.join("config.lua")
    };

    super::config::require_config(&lua, config_path)?;
    // let conductor = super::conductor::Conductor::new();
    // conductor.conduct()?;
    Ok(())
}

pub(super) fn generate(generate: &Generate) -> Result<()> {
    let file_path = generate.file_path.clone().unwrap_or("./templar.lua".into());
    super::config::api::gen_lua_wrapper(&file_path)?;
    Ok(())
}
