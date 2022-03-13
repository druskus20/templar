pub mod api; // TODO: Make this pub(super) once examples/ is not required
pub(super) mod rule;

use crate::hashmap;

use anyhow::Result;
use rlua::prelude::*;
use std::{collections::HashMap, env, path::PathBuf};

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub(super) struct TemplarConfig {
    pub rules: Vec<rule::Rule>,
    //pub engine_args: EngineArgs,
}

// TODO:
pub(super) struct EngineArgs {}

impl<'lua> FromLua<'lua> for TemplarConfig {
    fn from_lua(lua_value: rlua::Value<'lua>, _: rlua::Context<'lua>) -> rlua::Result<Self> {
        if let LuaValue::Table(lua_table) = lua_value {
            Ok(TemplarConfig {
                rules: lua_table.get("rules")?,
            })
        } else {
            Err(rlua::Error::external("Expected config to be a lua table"))
        }
    }
}

impl<'lua> ToLua<'lua> for TemplarConfig {
    fn to_lua(self, lua: rlua::Context<'lua>) -> rlua::Result<LuaValue<'lua>> {
        let hashmap: HashMap<&str, LuaValue> = hashmap!(
            "rule" => self.rules.to_lua(lua)?,
        );
        Ok(LuaValue::Table(LuaContext::create_table_from(
            lua, hashmap,
        )?))
    }
}

// TODO:
pub fn load_config(lua: &Lua, config_file: PathBuf) -> Result<()> {
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
        load_config(&lua, config_path).unwrap();
    }
}
