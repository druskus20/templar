use std::{fmt::Debug, path::PathBuf, rc::Rc};

use super::directive::Generator;

use super::parser;
use anyhow::Result;

#[derive(Debug, Clone)]
pub(super) struct Template {
    //pub settings
    pub blocks: Vec<Rc<dyn Generator>>,
}

impl TryFrom<PathBuf> for Template {
    type Error = anyhow::Error;

    fn try_from(value: PathBuf) -> Result<Self> {
        let file_contents = std::fs::read_to_string(value)?;
        parser::parse_template(&file_contents)
    }
}
