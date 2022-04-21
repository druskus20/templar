use anyhow::Result;
use glob::glob;
use std::path::{Path, PathBuf};

use crate::config::{rawconfig::RawConfig, rawrule::RawRule};

#[derive(Clone, Debug)]
pub(crate) struct Config {
    pub rules: Vec<Rule>,
    pub dest_base: PathBuf,
    //pub engine_args: EngineArgs,
}

impl Config {
    pub(crate) fn from_raw_config(raw_config: RawConfig) -> Result<Self> {
        Ok(Config {
            rules: raw_config
                .rules
                .into_iter()
                .map(|raw_rule| Rule::from_raw_rule(raw_rule))
                .collect::<Result<Vec<_>>>()?,
            dest_base: PathBuf::from(raw_config.dest_base),
            //engine_args: raw_config.engine_args,
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        let dest_base = PathBuf::from(".")
            .canonicalize()
            .unwrap_or_else(|e| panic!("Could not canonicalize current directory. {}", e));

        Config {
            rules: vec![],
            dest_base,
            //engine_args: EngineArgs::default(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub(crate) struct Rule {
    pub id: String, // Unique identifier
    pub targets: Vec<PathBuf>,
    pub rules: Vec<Rule>,
    pub basepath: PathBuf,
}

impl Rule {
    // TODO: Clean up this mess / test
    // TODO: This is all relying on PathBuf. Should be changed in somw way, probably. We shouldnt rely on PathBuf until its
    // time to call engine.run()
    pub(super) fn from_raw_rule(raw_rule: RawRule) -> Result<Self> {
        let rules = raw_rule
            .rules
            .into_iter()
            .map(|raw_rule| Rule::from_raw_rule(raw_rule))
            .collect::<Result<Vec<_>>>()?;

        let basepath: String = raw_rule.basepath;

        let children_targets = rules
            .iter()
            .flat_map(|r| r.targets.clone())
            .collect::<Vec<_>>();

        let mut targets =
            calc_targets(raw_rule.targets.clone(), basepath.clone()).map_err(|err| {
                rlua::Error::FromLuaConversionError {
                    to: "Rule",
                    from: "LuaValue",
                    message: Some(err.to_string()),
                }
            })?;

        let targets = targets
            .drain_filter(|t| !children_targets.contains(t))
            .collect();

        let id = raw_rule.id;

        Ok(Rule {
            id,
            targets,
            rules,
            basepath: basepath.into(),
        })
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
