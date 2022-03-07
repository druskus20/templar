use std::{fmt::Debug, path::PathBuf, rc::Rc};

use super::directive::Generator;

use super::parser::{self, ParserConfig};
use anyhow::Result;

pub(super) type DynGenerator = Rc<dyn Generator>;

#[derive(Debug, Clone)]
pub(crate) struct Template {
    //pub settings
    blocks: Vec<DynGenerator>,
}

impl Template {
    pub(crate) fn parse_path(config: &ParserConfig, template_path: PathBuf) -> Result<Self> {
        let file_contents = std::fs::read_to_string(template_path)?;
        Self::parse_str(config, &file_contents)
    }

    pub(crate) fn parse_str(config: &ParserConfig, template_str: &str) -> Result<Self> {
        match parser::parse_template_str(config, template_str) {
            Ok((_, blocks)) => Ok(Template { blocks }),
            Err(e) => anyhow::bail!("{}", e), // Rethrow the error (lifetimes stuff)
        }
    }

    pub(crate) fn process(&self) -> Result<String> {
        let mut output = String::new();
        rlua::Lua::new().context(|lua_context| -> Result<()> {
            for block in &self.blocks {
                let block_output = block.generate(&lua_context)?;
                output.push_str(block_output.as_str());
            }
            Ok(())
        })?;
        Ok(output)
    }
}
