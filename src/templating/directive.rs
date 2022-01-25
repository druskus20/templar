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
    pub condition: String,
    pub blocks: Vec<Rc<dyn Generator>>,
}

impl Generator for If {
    fn run(&self) -> Result<&str> {
        // TODO:
        Ok(self.condition.as_str())
    }
}

#[derive(Debug, Clone)]
pub(super) struct IfElse {
    pub condition: String,
    pub if_blocks: Vec<Rc<dyn Generator>>,
    pub else_blocks: Vec<Rc<dyn Generator>>,
}

impl Generator for IfElse {
    fn run(&self) -> Result<&str> {
        // TODO:
        Ok(self.condition.as_str())
    }
}

#[derive(Debug, Clone)]
pub(super) struct Include {
    pub path: String,
}

impl Generator for Include {
    fn run(&self) -> Result<&str> {
        // TODO:
        Ok(self.path.as_str())
    }
}
