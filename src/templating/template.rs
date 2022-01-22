use std::{collections::HashMap, path::PathBuf};

use super::{
    directive::{BlockDirective, LineDirective},
    parser,
};
use anyhow::{private::kind::BoxedKind, Result};

#[derive(Debug, Clone)]
pub(super) struct Template {
    //pub settings
    pub blocks: Vec<TemplateBlock>,
}

impl TryFrom<PathBuf> for Template {
    type Error = anyhow::Error;

    fn try_from(value: PathBuf) -> Result<Self> {
        let file_contents = std::fs::read_to_string(value)?;
        parser::parse_template(&file_contents)
    }
}

#[derive(Debug, Clone)]
pub(super) enum TemplateBlock {
    Text(String),
    BlockDirective(TemplateDirectiveBlock),
    LineDirective(TemplateDirectiveLine),
    // ...
}

trait Generator {
    fn run(&self) -> Result<&str>;
}

impl Generator for TemplateBlock {
    fn run(&self) -> Result<&str> {
        match self {
            TemplateBlock::Text(text) => Ok(text),
            TemplateBlock::BlockDirective(directive) => directive.run(),
            TemplateBlock::LineDirective(directive) => directive.run(),
        }
    }
}

// Clone is manually implemented for Box<dyn BlockDirective>
#[derive(Debug, Clone)]
pub(super) struct TemplateDirectiveBlock {
    pub directive: Box<dyn BlockDirective>,
    pub blocks: Vec<TemplateBlock>,
    //pub blocks: Vec<dyn Generator>,
}

#[derive(Debug, Clone)]
pub(super) struct TemplateDirectiveLine {
    pub directive: &'static dyn LineDirective,
}

impl Generator for TemplateDirectiveBlock {
    fn run(&self) -> Result<&str> {
        self.directive.run(self.blocks.clone()) // TODO: Clone
    }
}

impl Generator for TemplateDirectiveLine {
    fn run(&self) -> Result<&str> {
        self.directive.run()
    }
}
