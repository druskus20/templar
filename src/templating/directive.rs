use std::{fmt::Display, rc::Rc};

use anyhow::Result;
use rlua::prelude::*;
use std::fmt::Debug;

use super::parser::ParserConfig;

pub(super) trait Generator: Debug {
    fn generate(&self, lua_context: LuaContext) -> Result<&str> {
        Ok("generated!")
    }

    fn display(&self, c: ParserConfig) -> Result<&str> {
        todo!()
    }
}

// Text
impl<T> Generator for T
where
    T: AsRef<str> + Debug,
{
    fn generate(&self, _: LuaContext) -> Result<&str> {
        Ok(self.as_ref())
    }
}

#[derive(Debug, Clone)]
pub(super) struct If {
    pub condition: String,
    pub blocks: Vec<Rc<dyn Generator>>,
}

impl Generator for If {}

#[derive(Debug, Clone)]
pub(super) struct IfElse {
    pub condition: String,
    pub if_blocks: Vec<Rc<dyn Generator>>,
    pub else_blocks: Vec<Rc<dyn Generator>>,
}

impl Generator for IfElse {}

#[derive(Debug, Clone)]
pub(super) struct Include {
    pub path: String,
}

impl Generator for Include {}

#[derive(Debug, Clone)]
pub(super) struct Transform {
    pub transform: String,
    pub blocks: Vec<Rc<dyn Generator>>,
}

impl Generator for Transform {}
