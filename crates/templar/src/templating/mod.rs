mod directives;
mod parser;

use std::{fmt::Debug, io::Write, path::PathBuf};

use anyhow::Result;
use directives::DynDirective;
use parser::ParserConfig;

pub(crate) struct TemplateEngine {
    pub(crate) config: ParserConfig,
}

impl TemplateEngine {
    pub(crate) fn process(&self, template_path: PathBuf, output_path: PathBuf) -> Result<()> {
        let template = Template::load_from_path(&self.config, template_path)?;
        let output = template.process()?;
        std::fs::File::create(output_path)?.write_all(output.as_bytes())?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Template {
    // NOTE: Maybe abstract this into TemplateEngine (not really *the* engine tho), Template and TemplateConfig?
    //  relevant only if we want to support multiple engines (and make this its own create)
    blocks: Vec<DynDirective>,
    parser_config: Option<ParserConfig>,
}

impl Template {
    fn load_from_path(config: &ParserConfig, template_path: PathBuf) -> Result<Self> {
        let file_contents = std::fs::read_to_string(template_path)?;
        Self::from_str(config, &file_contents)
    }

    fn from_str(config: &ParserConfig, template_str: &str) -> Result<Self> {
        // TODO: Get the ParserConfig from the template file if possible, otherwise use the argument
        //      config = parse_config(template_str);
        match parser::parse_template_str(config, template_str) {
            Ok((_, blocks)) => Ok(Template {
                parser_config: Some(config.clone()),
                blocks,
            }),
            Err(e) => anyhow::bail!("{}", e), // Rethrow the error (lifetimes stuff)
        }
    }

    fn process(&self) -> Result<String> {
        let mut output = String::new();
        rlua::Lua::new().context(|lua_context| -> Result<()> {
            for block in &self.blocks {
                let block_output = block.generate(&lua_context)?;
                output.push_str(block_output.as_str());
            }
            Ok(())
        })?;
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::parser;
    use super::Template;
    use indoc::indoc;

    #[test]
    fn test_templar() {
        let config = parser::ParserConfig {
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

        let t = Template::from_str(&config, template_str).unwrap();
        let _ = t.process().unwrap();
        //println!("{}", r);

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

        let t = Template::from_str(&config, template_str).unwrap();
        let _ = t.process().unwrap();
    }
}
