use std::rc::Rc;

use anyhow::Result;
use rlua::prelude::*;
use std::fmt::Debug;

use super::parser::ParserConfig;

pub(super) type DynDirective = Rc<dyn Directive>;

pub(super) trait Directive: Debug {
    /* Generates a String from a Directive. */
    // NOTE: Possibly store ParserConfig inside Include and pass it from the parser?
    // NOTE: Possibly lua_context might be handled differently once I figure out how to to scopes
    fn generate(&self, parser_config: &ParserConfig, lua_context: &LuaContext) -> Result<String>;

    // NOTE: Might be sensible to put this method in ParserConfig and possibly add another trait?
    // idk lets keep it simple for now
    fn _display(&self, _parser_config: &ParserConfig) -> Result<String> {
        unimplemented!()
    }
}

// Text
//impl<T> Generator for T
//where
//    T: AsRef<str> + Debug,
//{
//    fn generate(&self, _: &LuaContext) -> Result<String> {
//        Ok(self.as_ref().to_string())
//    }
//}

impl Directive for String {
    fn generate(&self, _: &ParserConfig, _: &LuaContext) -> Result<String> {
        Ok(self.clone())
    }
}

impl Directive for &str {
    fn generate(&self, _: &ParserConfig, _: &LuaContext) -> Result<String> {
        Ok(self.to_string())
    }
}

#[derive(Debug, Clone)]
pub(super) struct If {
    pub condition: String,
    pub blocks: Vec<DynDirective>,
}

impl Directive for If {
    fn generate(&self, parser_config: &ParserConfig, lua_context: &LuaContext) -> Result<String> {
        let condition_result = lua_context.load(&self.condition).eval::<bool>()?;
        if condition_result {
            self.blocks.generate(parser_config, lua_context)
        } else {
            Ok("".to_string())
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct IfElse {
    pub condition: String,
    pub if_blocks: Vec<DynDirective>,
    pub else_blocks: Vec<DynDirective>,
}

impl Directive for IfElse {
    fn generate(&self, parser_config: &ParserConfig, lua_context: &LuaContext) -> Result<String> {
        let condition_result = lua_context.load(&self.condition).eval::<bool>()?;
        if condition_result {
            self.if_blocks.generate(parser_config, lua_context)
        } else {
            self.else_blocks.generate(parser_config, lua_context)
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct Include {
    pub path: String,
}

impl Directive for Include {
    fn generate(&self, parser_config: &ParserConfig, _lua_context: &LuaContext) -> Result<String> {
        let str = super::Template::load_from_path(parser_config, (&self.path).into())?
            .process(&parser_config)?;
        Ok(str)
    }
}

#[derive(Debug, Clone)]
pub(super) struct Transform {
    pub input_name: String,
    pub transform: String,
    pub blocks: Vec<DynDirective>,
}

impl Directive for Transform {
    fn generate(&self, parser_config: &ParserConfig, lua_context: &LuaContext) -> Result<String> {
        let blocks = self.blocks.generate(parser_config, lua_context)?;
        lua_context.globals().set(self.input_name.clone(), blocks)?;
        let r = lua_context.load(&self.transform).eval::<String>()?;
        lua_context.globals().set(self.input_name.clone(), LuaNil)?;
        Ok(r)
    }
}

impl Directive for Vec<DynDirective> {
    fn generate(&self, parser_config: &ParserConfig, lua_context: &LuaContext) -> Result<String> {
        let mut result = String::new();
        for block in self {
            result.push_str(&block.generate(parser_config, lua_context)?);
        }
        Ok(result.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::super::parser::ParserConfig;
    use super::*;
    use rlua::prelude::Lua;

    lazy_static::lazy_static! {
        static ref PARSER_CONFIG: ParserConfig = ParserConfig::default();
    }

    #[test]
    fn test_directive_str() {
        let parser_config = &PARSER_CONFIG;
        let directive = "some text";
        Lua::new().context(|lua_context| {
            let result = directive.generate(parser_config, &lua_context).unwrap();
            let expected = "some text".to_string();
            assert_eq!(result, expected);
        });
    }

    #[test]
    fn test_directive_string() {
        let parser_config = &PARSER_CONFIG;
        let directive = "some text".to_string();
        Lua::new().context(|lua_context| {
            let result = directive.generate(parser_config, &lua_context).unwrap();
            let expected = "some text".to_string();
            assert_eq!(result, expected);
        });
    }

    #[test]
    fn test_directive_if() {
        let parser_config = &PARSER_CONFIG;
        let directive_true = If {
            condition: "true".to_string(),
            blocks: vec![Rc::new("some text".to_string())],
        };
        let directive_false = If {
            condition: "false".to_string(),
            blocks: vec![Rc::new("some text".to_string())],
        };
        Lua::new().context(|lua_context| {
            let result = directive_true
                .generate(parser_config, &lua_context)
                .unwrap();
            let expected = "some text".to_string();
            assert_eq!(result, expected);
            let result = directive_false
                .generate(parser_config, &lua_context)
                .unwrap();
            let expected = "".to_string();
            assert_eq!(result, expected);
        });
    }

    #[test]
    fn test_directive_ifelse() {
        let parser_config = &PARSER_CONFIG;
        let directive_true = IfElse {
            condition: "true".to_string(),
            if_blocks: vec![Rc::new("some text".to_string())],
            else_blocks: vec![Rc::new("some more text".to_string())],
        };
        let directive_false = IfElse {
            condition: "false".to_string(),
            if_blocks: vec![Rc::new("some text".to_string())],
            else_blocks: vec![Rc::new("some more text".to_string())],
        };
        Lua::new().context(|lua_context| {
            let result = directive_true
                .generate(parser_config, &lua_context)
                .unwrap();
            let expected = "some text".to_string();
            assert_eq!(result, expected);
            let result = directive_false
                .generate(parser_config, &lua_context)
                .unwrap();
            let expected = "some more text".to_string();
            assert_eq!(result, expected);
        });
    }

    #[test]
    fn test_directive_include() {
        use std::io::Write;
        let root = tempdir::TempDir::new("test_directive_include").unwrap();
        let path = root.path().join("test_directive_include.lua");
        let file_contents = indoc::indoc!(
            r#"
            some text
            "#
        );
        let mut file = std::fs::File::create(&path).unwrap();
        file.write(file_contents.as_bytes()).unwrap();

        let parser_config = &PARSER_CONFIG;
        let directive = Include {
            path: path.to_string_lossy().to_string(),
        };
        Lua::new().context(|lua_context| {
            let result = directive.generate(parser_config, &lua_context).unwrap();
            let expected = "some text\n".to_string();
            assert_eq!(result, expected);
        });
    }

    #[test]
    fn test_directive_transform() {
        let parser_config = &PARSER_CONFIG;
        let directive = Transform {
            input_name: "input".to_string(),
            transform: "input:gsub(\"RED\", \"#FF0000\")".to_string(),
            blocks: vec![Rc::new("some text in RED".to_string())],
        };
        Lua::new().context(|lua_context| {
            let result = directive.generate(parser_config, &lua_context).unwrap();
            let expected = "some text in #FF0000".to_string();
            assert_eq!(result, expected);
        });
    }
}
