use std::{collections::HashMap, path::PathBuf, rc::Rc};

use super::{
    directive::{BlockDirective, LineDirective},
    parser,
};
use anyhow::{private::kind::BoxedKind, Result};

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

pub(super) trait Generator: std::fmt::Debug {
    fn run(&self) -> Result<&str>;
}

impl<T> Generator for T
where
    T: AsRef<str> + std::fmt::Debug + Clone + 'static,
{
    fn run(&self) -> Result<&str> {
        Ok(self.as_ref())
    }
}

// Clone is manually implemented for Box<dyn BlockDirective>
#[derive(Debug, Clone)]
pub(super) struct TemplateDirectiveBlock {
    pub directive: Rc<dyn BlockDirective>,
    pub blocks: Vec<Rc<dyn Generator>>,
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
