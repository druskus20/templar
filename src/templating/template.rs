use std::{collections::HashMap, path::PathBuf};

use super::{
    directive::{BlockDirective, LineDirective},
    parser,
};
use anyhow::Result;

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

//impl std::fmt::Debug for dyn TemplateBlockTRAIT {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        write!(f, "{}", "woo")
//        // TODO:
//    }
//}

#[derive(Debug, Clone)]
pub(super) enum TemplateBlock {
    Text(String),
    BlockDirective(TemplateBlockDirective),
    LineDirective(TemplateLineDirective),
    // ...
}

impl TemplateBlock {
    pub(super) fn run(&self) -> Result<&str> {
        match self {
            TemplateBlock::Text(text) => Ok(text),
            TemplateBlock::BlockDirective(directive) => directive.run(),
            TemplateBlock::LineDirective(directive) => directive.run(),
        }
    }
}

#[derive(Debug)]
pub(super) struct TemplateBlockDirective {
    pub directive: Box<dyn BlockDirective>,
    pub blocks: Vec<TemplateBlock>, // TODO: Lifetime shit?
}

impl Clone for TemplateBlockDirective {
    fn clone(&self) -> Self {
        TemplateBlockDirective {
            directive: self.directive.clone(),
            blocks: self.blocks.clone(),
        }
    }
}

impl TemplateBlockDirective {
    pub(super) fn run(&self) -> Result<&str> {
        self.directive.run(self.blocks.clone()) // TODO: Clone
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct TemplateLineDirective {
    pub directive: &'static dyn LineDirective,
}

impl TemplateLineDirective {
    pub(super) fn run(&self) -> Result<&str> {
        self.directive.run()
    }
}
