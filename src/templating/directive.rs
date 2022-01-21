use super::template::{Template, TemplateBlock, TemplateBlockDirective, TemplateLineDirective};
use anyhow::Result;

pub(super) trait BlockDirective: DirectiveClone {
    fn run(&self, contents: Vec<TemplateBlock>) -> Result<&str>;
}

// ----------
//Black magic to make the let the compiler clone a Box<dyn BlockDirective>
//https://stackoverflow.com/questions/30353462/how-to-clone-a-struct-storing-a-boxed-trait-object
pub(super) trait DirectiveClone {
    fn clone_box(&self) -> Box<dyn BlockDirective>;
}

impl<T> DirectiveClone for T
where
    // NOTE: This is super weird.
    // TODO: Does static even make sense?
    T: BlockDirective + 'static + Clone,
{
    fn clone_box(&self) -> Box<dyn BlockDirective> {
        Box::new((*self).clone()) // We can do this because of the bound Clone
    }
}

impl Clone for Box<dyn BlockDirective> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
// ----------

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
