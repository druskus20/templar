use super::rawrule::RawRule;
use anyhow::Result;
use rlua::prelude::*;
use std::{env, path::PathBuf};

#[derive(Clone, Default, Debug)]
pub(crate) struct RawConfig {
    pub rules: Vec<RawRule>,
    pub dest_base: String,
}

// TODO:
pub(super) struct EngineArgs {}

// TODO:
pub fn require_config(lua: &Lua, config_file: PathBuf) -> Result<()> {
    let config_filename = config_file
        .file_stem()
        .ok_or_else(|| anyhow::anyhow!("No config file name"))?
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Config file name is not valid UTF-8"))?;

    let config_path = config_file
        .parent()
        .ok_or_else(|| anyhow::anyhow!("No config file path"))?;

    // NOTE: This might be problematic
    env::set_current_dir(config_path)?;

    lua.context(|lua_context| {
        lua_context
            .load(&format!(r#"require "{}""#, config_filename))
            .exec()?;
        LuaResult::Ok(())
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    use indoc::indoc;
    use std::fs::File;
    use tempdir::TempDir;

    #[test]
    fn test_run_config() {
        let root = TempDir::new("test_run_config");
        let root = root.ok().expect("Should have created a temp directory");

        let base_path = root.path().join("base");
        let config_path = root.path().join("config.lua");

        std::fs::create_dir(base_path).unwrap();

        let config = indoc!(
            r#"
            -- config.lua
            local config = {}
            config.something = {}
            return config
            "#
        );

        File::create(&config_path)
            .unwrap()
            .write(config.as_bytes())
            .unwrap();

        // Test starts here
        let lua = Lua::new();
        require_config(&lua, config_path).unwrap();
    }
}
