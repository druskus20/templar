use super::template::{Template, TemplateBlock, TemplateDirectiveBlock, TemplateDirectiveLine};
use anyhow::Result;

pub(super) trait BlockDirective: DirectiveClone + std::fmt::Debug {
    fn run(&self, contents: Vec<TemplateBlock>) -> Result<&str>;
}

// ----------
// Black magic to make the let the compiler clone a Box<dyn BlockDirective>
//  https://stackoverflow.com/questions/30353462/how-to-clone-a-struct-storing-a-boxed-trait-object
pub(super) trait DirectiveClone {
    fn clone_box(&self) -> Box<dyn BlockDirective>;
}

impl<T> DirectiveClone for T
where
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

// Alternatives I wish worked
//
// impl<T> Clone for T
// where
//     T: BlockDirective + 'static + Clone,
// {
//     fn clone(&self) -> Self {
//         (*self).clone() // We can do this because of the bound Clone
//     }
// }
//
// No because T is not being used? Self is weird?
//
// -----
//
// pub directive: Box<dyn BlockDirective + Clone>,
//
// No because Clone is not object safe
//
// -----
//
// pub(super) trait BlockDirective: Clone + 'static {
//     fn run(&self, contents: Vec<TemplateBlock>) -> Result<&str>;
// }
//
// No because Clone is not object safe
//
// ----------

pub(super) trait LineDirective: std::fmt::Debug {
    fn run(&self) -> Result<&str>;
}

#[derive(Debug, Clone)]
pub(super) struct DoNothing {
    pub text: String,
}

impl BlockDirective for DoNothing {
    fn run(&self, contents: Vec<TemplateBlock>) -> Result<&str> {
        Ok(&self.text)
    }
}

impl LineDirective for DoNothing {
    fn run(&self) -> Result<&str> {
        Ok(&self.text)
    }
}
