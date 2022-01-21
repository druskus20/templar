use super::template::{Template, TemplateBlock, TemplateBlockDirective, TemplateLineDirective};
use anyhow::Result;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) enum BlockDirective {
    DoNothing(DoNothingBlock),
    // If(IfDirective),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) enum LineDirective {
    DoNothing(DoNothingLine),
    // Include(IncludeDirective),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct DoNothingLine {
    pub text: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct DoNothingBlock {
    pub text: String,
}

trait BlockDirectiveExt {
    fn run(&self, contents: &Vec<TemplateBlock>) -> Result<&str>;
}

impl BlockDirectiveExt for BlockDirective {
    fn run(&self, contents: &Vec<TemplateBlock>) -> Result<&str> {
        match &self {
            BlockDirective::DoNothing(d) => d.run(contents),
        }
    }
}

impl BlockDirectiveExt for DoNothingBlock {
    fn run(&self, _contents: &Vec<TemplateBlock>) -> Result<&str> {
        Ok(&self.text)
    }
}

trait LineDirectiveExt {
    fn run(&self) -> Result<&str>;
}

impl LineDirectiveExt for LineDirective {
    fn run(&self) -> Result<&str> {
        match &self {
            LineDirective::DoNothing(d) => d.run(),
        }
    }
}

impl LineDirectiveExt for DoNothingLine {
    fn run(&self) -> Result<&str> {
        Ok(&self.text)
    }
}
