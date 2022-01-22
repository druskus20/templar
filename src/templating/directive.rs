use std::rc::Rc;

use anyhow::Result;
use std::fmt::Debug;

pub(super) trait Generator: Debug {
    fn run(&self) -> Result<&str>;
}

// Text
impl<T> Generator for T
where
    T: AsRef<str> + Debug,
{
    fn run(&self) -> Result<&str> {
        Ok(self.as_ref())
    }
}

#[derive(Debug, Clone)]
pub(super) struct If {
    pub blocks: Vec<Rc<dyn Generator>>,
}

impl Generator for If {
    fn run(&self) -> Result<&str> {
        // TODO:
        todo!()
    }
}

#[derive(Debug, Clone)]
pub(super) struct DoNothing {
    pub text: String,
}

impl Generator for DoNothing {
    fn run(&self) -> Result<&str> {
        Ok(self.text.as_str())
    }
}

#[derive(Debug, Clone)]
pub(super) struct UselessBlockWithText {
    pub text: String,
    pub blocks: Vec<Rc<dyn Generator>>,
}

impl Generator for UselessBlockWithText {
    fn run(&self) -> Result<&str> {
        Ok(self.text.as_str())
    }
}
