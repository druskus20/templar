use std::{collections::HashMap, path::PathBuf};

use super::{
    directive::{BlockDirective, LineDirective},
    parser,
};
use anyhow::Result;

#[derive(Debug, Clone)]
pub(super) struct Template {
    //pub settings
    pub blocks: Vec<TemplateBlock>, // TODO: Maybe do this with traits so that I dont need 200 nested enums
}

impl TryFrom<PathBuf> for Template {
    type Error = anyhow::Error;

    fn try_from(value: PathBuf) -> Result<Self> {
        let file_contents = std::fs::read_to_string(value)?;
        parser::parse_template(&file_contents)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) enum TemplateBlock {
    Text(String),
    BlockDirective(TemplateBlockDirective),
    LineDirective(TemplateLineDirective),
    // ...
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct TemplateBlockDirective {
    pub directive: BlockDirective,
    pub blocks: Vec<TemplateBlock>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct TemplateLineDirective {
    pub directive: LineDirective,
}
