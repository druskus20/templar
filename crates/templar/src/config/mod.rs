pub(super) mod api; // TODO: Make this pub(super) once examples/ is not required
mod rawrule;
pub(super) mod rule;

use anyhow::Result;
use rlua::prelude::*;
use std::{env, path::PathBuf, rc::Rc};
use vfs::{FileSystem, VfsPath};

#[derive(Clone, Debug)]
pub(crate) struct TemplarConfig {
    pub rules: Vec<rule::Rule>,
    pub dest_base: VfsPath,
    //pub engine_args: EngineArgs,

    // NOTE: Not using DynClone for now
    pub fs: Rc<dyn FileSystem>,
}

impl TemplarConfig {
    pub(crate) fn from_raw_config(raw_config: RawConfig, fs: impl FileSystem) -> Result<Self> {
        let fs = Rc::new(fs);
        Ok(TemplarConfig {
            rules: raw_config
                .rules
                .into_iter()
                .map(|raw_rule| rule::Rule::from_raw_rule(raw_rule, fs.clone()))
                .collect::<Result<Vec<_>>>()?,
            dest_base: PathBuf::from(raw_config.dest_base),
            fs,
            //engine_args: raw_config.engine_args,
        })
    }
}

impl Default for TemplarConfig {
    fn default() -> Self {
        let dest_base = PathBuf::from(".")
            .canonicalize()
            .unwrap_or_else(|e| panic!("Could not canonicalize current directory. {}", e));

        TemplarConfig {
            rules: vec![],
            dest_base,
            //engine_args: EngineArgs::default(),
            fs: Rc::new(vfs::MemoryFS::new()),
        }
    }
}

#[derive(Clone, Default, Debug)]
pub(crate) struct RawConfig {
    pub rules: Vec<rawrule::RawRule>,
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
