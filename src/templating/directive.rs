use std::rc::Rc;

use super::template::{Generator, Template, TemplateDirectiveBlock, TemplateDirectiveLine};
use anyhow::Result;

pub(super) trait BlockDirective: std::fmt::Debug {
    fn run(&self, contents: Vec<Rc<dyn Generator>>) -> Result<&str>;
}

pub(super) trait LineDirective: std::fmt::Debug {
    fn run(&self) -> Result<&str>;
}

#[derive(Debug, Clone)]
pub(super) struct DoNothing {
    pub text: String,
}

impl BlockDirective for DoNothing {
    fn run(&self, contents: Vec<Rc<dyn Generator>>) -> Result<&str> {
        Ok(&self.text)
    }
}

impl LineDirective for DoNothing {
    fn run(&self) -> Result<&str> {
        Ok(&self.text)
    }
}
