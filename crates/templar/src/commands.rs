use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use super::opt::{Generate, Run};
use crate::conductor::engine::Engine;
use crate::config::TemplarConfig;
use anyhow::Result;
use rlua::Lua;

pub(super) fn run(run: &Run) -> Result<()> {
    // Drop the arked config at the end...?
    let config = TemplarConfig::default();
    let arked_config = Arc::new(Mutex::new(config)); // Cant clone here, because I dont want a copy
    {
        // TODO: @important Captures the Arc<Mutex<TemplarConfig>>. If I want to
        // reuse the lua context for my engine (i.e. global lua variables), I will
        // wneed to clone it.
        // NOTE: This should not be a problem, as global variables should be instead implemented via EngineArgs, using some sort of
        // abstraction layer over the Engine, so that engines that dont use lua can be implemented

        let lua = Lua::new();
        super::config::api::register_lua_api(arked_config.clone(), &lua)?;

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
    }

    // We know that, at this point, the other references to the inner contents
    // of the Arc have been dropped, as the Lua context is dropped. Therefore it
    // is safe to unwrap
    let config = Arc::try_unwrap(arked_config)
        .unwrap_or_else(|e| panic!("Failed to unwrap Arc for the config: {:?}", e))
        .into_inner()
        .unwrap_or_else(|e| panic!("Failed to unwrap Mutex for the config: {:?}", e));

    // TODO: At the moment all of these are being hardcoded
    let parser_config = super::conductor::trebuchet::parser::ParserConfig::default();
    let engine = super::conductor::trebuchet::Trebuchet::new(parser_config);
    let _conductor = super::conductor::Conductor::new(Box::new(engine), dbg!(config));
    // conductor.conduct()?;
    Ok(())
}

pub(super) fn generate(generate: &Generate) -> Result<()> {
    let file_path = generate.file_path.clone().unwrap_or("./templar.lua".into());
    super::config::api::gen_lua_wrapper(&file_path)?;
    Ok(())
}
