use std::{io::Write, path::Path};

use crate::config::TemplarConfig;

use anyhow::Result;
use engine::Engine;

mod engine;
mod trebuchet;

struct Conductor {
    engine: Box<dyn Engine>,
    config: TemplarConfig,
}

impl Conductor {
    pub(super) fn new(engine: Box<dyn Engine>, config: TemplarConfig) -> Self {
        Conductor {
            engine: engine,
            config,
        }
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
}
