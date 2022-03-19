use std::path::PathBuf;

use anyhow::Result;
use dyn_clone::DynClone;
use rlua::prelude::*;
use std::fmt::Debug;

use super::parser::ParserConfig;
use super::Engine;

pub(super) type DynDirective = Box<dyn Directive>;

dyn_clone::clone_trait_object!(Directive);

pub(super) trait Directive: Debug + DynClone {
    /* Generates a String from a Directive. */
    // NOTE: Possibly store ParserConfig inside Include and pass it from the parser?
    // NOTE: Possibly lua_context might be handled differently once I figure out how to to scopes
    fn generate(&self, lua_context: &LuaContext) -> Result<String>;

    // NOTE: Might be sensible to put this method in ParserConfig and possibly add another trait?
    // idk lets keep it simple for now
    fn _display(&self, _parser_config: &ParserConfig) -> Result<String> {
        unimplemented!()
    }
}

/*
 * TODO:
 * Turn Vec<DynDirective> into a type alias for Template.
 * Make a Parser struct, with methods to parse a Vec<DynDirective>
 */

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
    fn generate(&self, _: &LuaContext) -> Result<String> {
        Ok(self.clone())
    }
}

impl Directive for &str {
    fn generate(&self, _: &LuaContext) -> Result<String> {
        Ok(self.to_string())
    }
}

#[derive(Debug, Clone)]
pub(super) struct If {
    pub condition: String,
    pub blocks: Vec<DynDirective>,
}

impl Directive for If {
    fn generate(&self, lua_context: &LuaContext) -> Result<String> {
        let condition_result = lua_context.load(&self.condition).eval::<bool>()?;
        if condition_result {
            self.blocks.generate(lua_context)
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
    fn generate(&self, lua_context: &LuaContext) -> Result<String> {
        let condition_result = lua_context.load(&self.condition).eval::<bool>()?;
        if condition_result {
            self.if_blocks.generate(lua_context)
        } else {
            self.else_blocks.generate(lua_context)
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct Include {
    pub path: String,
    pub parser_config: ParserConfig, // TODO: Possibly use a reference
}

impl Directive for Include {
    fn generate(&self, _lua_context: &LuaContext) -> Result<String> {
        // TODO: Paths are handled by the conductor. Including directly from here is hacky
        let engine = super::Trebuchet::new(self.parser_config.clone());
        let path = PathBuf::from(self.path.clone());
        let template_str = std::fs::read_to_string(path.as_path())?;
        engine.process_template_str(template_str.as_str())
    }
}

#[derive(Debug, Clone)]
pub(super) struct Transform {
    pub input_name: String,
    pub transform: String,
    pub blocks: Vec<DynDirective>,
}

impl Directive for Transform {
    fn generate(&self, lua_context: &LuaContext) -> Result<String> {
        let blocks = self.blocks.generate(lua_context)?;
        lua_context.globals().set(self.input_name.clone(), blocks)?;
        let r = lua_context.load(&self.transform).eval::<String>()?;
        lua_context.globals().set(self.input_name.clone(), LuaNil)?;
        Ok(r)
    }
}

impl Directive for Vec<DynDirective> {
    fn generate(&self, lua_context: &LuaContext) -> Result<String> {
        let mut result = String::new();
        for block in self {
            result.push_str(&block.generate(lua_context)?);
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
        let directive = "some text";
        Lua::new().context(|lua_context| {
            let result = directive.generate(&lua_context).unwrap();
            let expected = "some text".to_string();
            assert_eq!(result, expected);
        });
    }

    #[test]
    fn test_directive_string() {
        let directive = "some text".to_string();
        Lua::new().context(|lua_context| {
            let result = directive.generate(&lua_context).unwrap();
            let expected = "some text".to_string();
            assert_eq!(result, expected);
        });
    }

    #[test]
    fn test_directive_if() {
        let directive_true = If {
            condition: "true".to_string(),
            blocks: vec![Box::new("some text".to_string())],
        };
        let directive_false = If {
            condition: "false".to_string(),
            blocks: vec![Box::new("some text".to_string())],
        };
        Lua::new().context(|lua_context| {
            let result = directive_true.generate(&lua_context).unwrap();
            let expected = "some text".to_string();
            assert_eq!(result, expected);
            let result = directive_false.generate(&lua_context).unwrap();
            let expected = "".to_string();
            assert_eq!(result, expected);
        });
    }

    #[test]
    fn test_directive_ifelse() {
        let directive_true = IfElse {
            condition: "true".to_string(),
            if_blocks: vec![Box::new("some text".to_string())],
            else_blocks: vec![Box::new("some more text".to_string())],
        };
        let directive_false = IfElse {
            condition: "false".to_string(),
            if_blocks: vec![Box::new("some text".to_string())],
            else_blocks: vec![Box::new("some more text".to_string())],
        };
        Lua::new().context(|lua_context| {
            let result = directive_true.generate(&lua_context).unwrap();
            let expected = "some text".to_string();
            assert_eq!(result, expected);
            let result = directive_false.generate(&lua_context).unwrap();
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

        let parser_config: &ParserConfig = &PARSER_CONFIG;
        let directive = Include {
            path: path.to_string_lossy().to_string(),
            parser_config: parser_config.clone(),
        };
        Lua::new().context(|lua_context| {
            let result = directive.generate(&lua_context).unwrap();
            let expected = "some text\n".to_string();
            assert_eq!(result, expected);
        });
    }

    #[test]
    fn test_directive_transform() {
        let directive = Transform {
            input_name: "input".to_string(),
            transform: "input:gsub(\"RED\", \"#FF0000\")".to_string(),
            blocks: vec![Box::new("some text in RED".to_string())],
        };
        Lua::new().context(|lua_context| {
            let result = directive.generate(&lua_context).unwrap();
            let expected = "some text in #FF0000".to_string();
            assert_eq!(result, expected);
        });
    }
}
