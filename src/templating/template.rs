use std::{collections::HashMap, path::PathBuf};

use super::{
    directive::{BlockDirective, LineDirective},
    parser,
};
use anyhow::{private::kind::BoxedKind, Result};

#[derive(Debug, Clone)]
pub(super) struct Template {
    //pub settings
    pub blocks: Vec<Box<dyn Generator>>,
}

impl TryFrom<PathBuf> for Template {
    type Error = anyhow::Error;

    fn try_from(value: PathBuf) -> Result<Self> {
        let file_contents = std::fs::read_to_string(value)?;
        parser::parse_template(&file_contents)
    }
}

pub(super) trait Generator: GeneratorClone + std::fmt::Debug {
    fn run(&self) -> Result<&str>;
}

// ---- Black magic to impl Clone for Generator
pub(super) trait GeneratorClone {
    fn clone_box(&self) -> Box<dyn Generator>;
}

impl<T> GeneratorClone for T
where
    T: Generator + 'static + Clone,
{
    fn clone_box(&self) -> Box<dyn Generator> {
        Box::new((*self).clone()) // We can do this because of the bound Clone
    }
}

impl Clone for Box<dyn Generator> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// ----

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
    pub directive: Box<dyn BlockDirective>,
    pub blocks: Vec<Box<dyn Generator>>,
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
