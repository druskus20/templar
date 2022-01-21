use super::template::{Template, TemplateBlock, TemplateBlockDirective, TemplateLineDirective};
use anyhow::Result;

pub(super) trait BlockDirective {
    fn run(&self, contents: Vec<TemplateBlock>) -> Result<&str>;
}

impl std::fmt::Debug for dyn BlockDirective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "TODO Implement Debug for BlockDirective")
        // TODO:
    }
}

pub(super) trait LineDirective {
    fn run(&self) -> Result<&str>;
}

impl std::fmt::Debug for dyn LineDirective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "TODO Implement Debug for LineDirective")
        // TODO:
    }
}

#[derive(Debug, Clone)]
pub(super) struct DoNothingLine {
    pub text: String,
}

impl LineDirective for DoNothingLine {
    fn run(&self) -> Result<&str> {
        Ok(&self.text)
    }
}

#[derive(Debug, Clone)]
pub(super) struct DoNothingBlock {
    pub text: String,
}

impl BlockDirective for DoNothingBlock {
    fn run(&self, contents: Vec<TemplateBlock>) -> Result<&str> {
        Ok(&self.text)
    }
}
