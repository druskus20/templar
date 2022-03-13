use std::io::Write;
use std::path::Path;

use anyhow::Result;

mod directives;
mod parser;
mod template;

use super::config::TemplarConfig; // TODO: I should not need this here, it is up to the director
use crate::config::rule::Rule;
use parser::ParserConfig;
use template::Template;

pub(super) struct Engine {
    // NOTE: This should probably be turn into some sort of EngineArgs struct, containing, ParserConfig
    //  among other things
    pub parser_config: ParserConfig,
    pub templar_config: TemplarConfig,
}

impl Engine {
    // Runs the engine over all the targets specified in the config
    pub(super) fn run(&self) {
        for rule in &self.templar_config.rules {
            self.run_rule_recursive(rule);
        }
    }

    // Recursive function to traverse the rule structure and process the templates
    fn run_rule_recursive(&self, rule: &Rule) {
        for r in &rule.rules {
            self.run_rule_recursive(r);
        }
        // Run with rule
        for target in &rule.targets {
            dbg!(target);
            // TODO: self.process_template_at(target, output_path)
        }
    }

    // Loads and then processes the template found at a given path, genereating the output file
    pub(super) fn process_template_at(
        &self,
        template_path: impl AsRef<Path>,
        output_path: impl AsRef<Path>,
    ) -> Result<()> {
        let template = Template::load_from_path(&self.parser_config, template_path)?;
        let output = template.process()?;
        std::fs::File::create(output_path)?.write_all(output.as_bytes())?;
        Ok(())
    }
}
