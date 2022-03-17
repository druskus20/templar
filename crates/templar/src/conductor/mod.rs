use std::{io::Write, path::Path};

use crate::config::{rule::Rule, TemplarConfig};

use anyhow::Result;
use engine::Engine;

mod engine;
mod trebuchet;

/*
 * TODO:
 * I probably need some sort of Rule stack that stores the hierarchy or rules
 * that are currently being evaluated, so that things like Include can resolve
 * paths relatively.
 */
#[derive(Clone)]
pub(super) struct Conductor {
    engine: Box<dyn Engine>,
    config: TemplarConfig,
}

impl Conductor {
    pub(super) fn new(engine: Box<dyn Engine>, config: TemplarConfig) -> Self {
        Conductor { engine, config }
    }

    pub(super) fn process_file_at(
        &self,
        template_path: impl AsRef<Path>,
        output_path: impl AsRef<Path>,
    ) -> Result<()> {
        let input = std::fs::read_to_string(template_path)?;
        let output = self.engine.run(input.as_str())?;
        std::fs::File::create(output_path)?.write_all(output.as_bytes())?;
        Ok(())
    }

    pub(super) fn conduct(&self) -> Result<()> {
        /* I need to handle
         *   - basepaths / relative paths
         *   - includes
         */
        for rule in &self.config.rules {
            self.process_rule(rule)?;
        }
        Ok(())
    }

    fn process_rule(&self, rule: &Rule) -> Result<()> {
        for rule in &rule.rules {
            self.process_rule(rule)?;
        }

        for target in &rule.targets {
            //self.process_file_at(target, self.config.dest_base.join(target))?;
            println!(
                "self.process_file_at({:?}, {:?})?",
                target,
                self.config.dest_base.join(target)
            );
            // TODO!
        }

        Ok(())
    }
}
