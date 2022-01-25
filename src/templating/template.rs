use std::{fmt::Debug, path::PathBuf, rc::Rc};

use super::directive::Generator;

use super::parser::{self, ParserConfig};
use anyhow::Result;

pub(super) type TemplateBlock = Rc<dyn Generator>;

#[derive(Debug, Clone)]
pub(super) struct Template {
    //pub settings
    pub blocks: Vec<Rc<dyn Generator>>,
}

impl Template {
    pub(super) fn generate(config: ParserConfig, path: PathBuf) -> Result<Self> {
        let file_contents = std::fs::read_to_string(path)?;
        match parser::parse_template_str(&config, &file_contents) {
            Ok((_, blocks)) => Ok(Template { blocks }),
            Err(e) => anyhow::bail!("{}", e), // Rethrow the error (lifetimes stuff)
        }
    }
}
