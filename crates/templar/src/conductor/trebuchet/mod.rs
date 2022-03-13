use self::parser::ParserConfig;
use super::engine::Engine;
use anyhow::Result;
use parser::Parser;

mod directives;
mod parser;

pub(super) struct Trebuchet {
    parser: Parser, // TODO: maybe this should be a reference? Includes create new Treckbuckets
}

impl Default for Trebuchet {
    fn default() -> Self {
        Trebuchet {
            parser: Parser {
                config: ParserConfig::default(),
            },
        }
    }
}

impl Trebuchet {
    // NOTE: This method should ideally be on the trait Engine, so the conductor can call it for any engine
    // It should also take EngineArgs instead of ParserConfig
    fn new(parser_config: ParserConfig) -> Self {
        Trebuchet {
            parser: Parser {
                config: parser_config,
            },
        }
    }

    fn process_template_str(&self, template_str: &str) -> Result<String> {
        let directives = self.parser.parse_template_str(template_str)?;
        let mut output = String::new();
        rlua::Lua::new().context(|lua_context| -> Result<()> {
            for directive in directives {
                let r = directive.generate(&lua_context)?;
                output.push_str(r.as_str());
            }
            Ok(())
        })?;
        Ok(output)
    }
}

impl Engine for Trebuchet {
    fn run(&self, input: &str) -> Result<String> {
        self.process_template_str(input)
    }
}

#[cfg(test)]
mod tests {
    use super::{parser::ParserConfig, Trebuchet};
    use indoc::indoc;

    #[test]
    fn test_trebuchet() {
        let config = ParserConfig {
            include: "include".to_string(),
            transform: "transform".to_string(),
            to: "to".to_string(),
            end: "end".to_string(),
            odelim: "<%".to_string(),
            cdelim: "%>".to_string(),
            ..Default::default()
        };

        let template_str = indoc!(
            r#"
                <% if "something" == "something" %>
                text
                <% if "something" == "NO" %>
                text2
                <% end %>
                <% end %>
                <% if "something" == "something" %>
                text3
                <% end %>
            "#
        );

        let trebuchet = Trebuchet::new(config.clone());
        let output = trebuchet.process_template_str(template_str).unwrap();
        let expected = indoc!(
            r#"
                text
                text3
            "#
        );
        assert_eq!(output, expected);

        let template_str = indoc!(
            r#"
                <% transform input %>
                local text = "wooo";
                return text;
                <% to %>
                text1
                text2
                text3
                <% end %>
            "#
        );

        let trebuchet = Trebuchet::new(config);
        let output = trebuchet.process_template_str(template_str).unwrap();
        let expected = "wooo".to_string();
        assert_eq!(output, expected);
    }
}
