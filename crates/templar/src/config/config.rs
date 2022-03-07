use anyhow::Result;
use glob::glob;
use rlua::prelude::{FromLua, LuaContext, LuaValue, ToLua};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::hashmap;

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub(super) struct Config {
    pub(super) rules: Vec<Rule>,
}

impl<'lua> FromLua<'lua> for Config {
    fn from_lua(lua_value: rlua::Value<'lua>, _: rlua::Context<'lua>) -> rlua::Result<Self> {
        if let LuaValue::Table(lua_table) = lua_value {
            Ok(Config {
                rules: lua_table.get("rules")?,
            })
        } else {
            Err(rlua::Error::external("Expected config to be a lua table"))
        }
    }
}

impl<'lua> ToLua<'lua> for Config {
    fn to_lua(self, lua: rlua::Context<'lua>) -> rlua::Result<LuaValue<'lua>> {
        let hashmap: HashMap<&str, LuaValue> = hashmap!(
            "rule" => self.rules.to_lua(lua)?,
        );
        Ok(LuaValue::Table(LuaContext::create_table_from(
            lua, hashmap,
        )?))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub(super) struct Rule {
    id: String, // Unique identifier
    targets: Vec<PathBuf>,
    rules: Vec<Rule>,
    basepath: PathBuf,
    /// This should not be used, its only for implementing ToLua
    raw_targets: String,
}

impl<'lua> FromLua<'lua> for Rule {
    fn from_lua(lua_value: rlua::Value<'lua>, _: rlua::Context<'lua>) -> rlua::Result<Self> {
        if let LuaValue::Table(lua_table) = lua_value {
            let rules: Vec<Rule> = lua_table.get("rules")?;
            let basepath: String = lua_table.get("basepath")?;

            // Calculate children targets and substract them from this rule's targets
            let children_targets = rules
                .iter()
                .flat_map(|r| r.targets.clone())
                .collect::<Vec<_>>();

            let raw_targets: String = lua_table.get("targets")?;

            let mut targets =
                calc_targets(raw_targets.clone(), basepath.clone()).map_err(|err| {
                    rlua::Error::FromLuaConversionError {
                        to: "Rule",
                        from: "LuaValue",
                        message: Some(err.to_string()),
                    }
                })?;

            let targets = targets
                .drain_filter(|t| !children_targets.contains(t))
                .collect();

            Ok(Rule {
                id: lua_table.get("id")?,
                targets,
                rules,
                basepath: basepath.into(),
                raw_targets,
            })
        } else {
            Err(rlua::Error::FromLuaConversionError {
                to: "Rule",
                from: "LuaValue",
                message: Some("Expected rule to be a lua table".to_string()),
            })
        }
    }
}

impl<'lua> ToLua<'lua> for Rule {
    fn to_lua(self, lua: rlua::Context<'lua>) -> rlua::Result<LuaValue<'lua>> {
        let hashmap: HashMap<&str, LuaValue> = hashmap!(
            "id" => self.id.to_lua(lua)?,
            "targets" => self.raw_targets.to_lua(lua)?,
            "rules" => self.rules.to_lua(lua)?,
            "basepath" => self.basepath.display().to_string().to_lua(lua)?,
        );
        Ok(LuaValue::Table(LuaContext::create_table_from(
            lua, hashmap,
        )?))
    }
}

fn calc_targets(path: String, basepath: String) -> Result<Vec<PathBuf>> {
    let home = std::env::var("HOME")?;
    let path = path.replace('~', home.as_str());

    // Concatenate basepath with path
    // TODO: Hacky
    let basepath = if basepath.ends_with('/') || basepath.is_empty() {
        basepath
    } else {
        format!("{}/", basepath)
    };

    let path = basepath + &path;
    let paths = glob(path.as_str())?;

    let mut targets = Vec::new();
    for path in paths {
        let path = path?;
        if path.is_dir() {
            targets.extend(expand_dir_rec(path)?);
        } else if path.is_file() {
            targets.push(std::fs::canonicalize(path)?);
        }
    }
    Ok(targets)
}

fn expand_dir_rec(dir: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let contents = std::fs::read_dir(dir)?;

    let mut targets = Vec::new();
    for entry in contents {
        let path = entry?.path();
        if path.is_dir() {
            targets.extend(expand_dir_rec(path)?);
        } else if path.is_file() {
            targets.push(std::fs::canonicalize(path)?);
        }
    }
    Ok(targets)
}

#[cfg(test)]
mod test {
    use std::fs::create_dir;
    use std::fs::File;
    use std::path::PathBuf;
    use tempdir::TempDir;

    #[test]
    fn test_calc_targets() {
        let root = TempDir::new("test_calc_targets");
        let root = root.ok().expect("Should have created a temp directory");

        let base_path = root.path().join("base");

        create_dir(&base_path).unwrap();
        create_dir(&base_path.join("aaa")).unwrap();
        File::create(&base_path.join("aaa/filea.txt")).unwrap();
        File::create(&base_path.join("aaa/fileaa.txt")).unwrap();
        create_dir(&base_path.join("aaa/bbb")).unwrap();
        File::create(&base_path.join("aaa/bbb/fileb.txt")).unwrap();
        create_dir(&base_path.join("aaa/bbb/ccc")).unwrap();
        File::create(&base_path.join("aaa/bbb/ccc/filec.txt")).unwrap();

        let mut targets =
            super::calc_targets("aaa/*".to_string(), base_path.display().to_string()).unwrap();

        let mut expected = vec![
            PathBuf::from(
                root.path()
                    .join("base/aaa/filea.txt")
                    .canonicalize()
                    .unwrap(),
            ),
            PathBuf::from(
                root.path()
                    .join("base/aaa/fileaa.txt")
                    .canonicalize()
                    .unwrap(),
            ),
            PathBuf::from(
                root.path()
                    .join("base/aaa/bbb/fileb.txt")
                    .canonicalize()
                    .unwrap(),
            ),
            PathBuf::from(
                root.path()
                    .join("base/aaa/bbb/ccc/filec.txt")
                    .canonicalize()
                    .unwrap(),
            ),
        ];

        targets.sort();
        expected.sort();
        assert_eq!(targets, expected);
    }
}
