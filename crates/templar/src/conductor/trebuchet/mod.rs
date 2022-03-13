use self::parser::ParserConfig;
use super::engine::Engine;

mod directives;
mod parser;
mod template;

pub(super) struct Trebuchet {
    parser_config: ParserConfig,
}

impl Trebuchet {
    // NOTE: This method should ideally be on the trait Engine, so the conductor can call it for any engine
    // It should also take EngineArgs instead of ParserConfig
    fn new(parser_config: ParserConfig) -> Self {
        Trebuchet { parser_config }
    }
}

impl Engine for Trebuchet {
    fn run(&self, input: &str) -> std::result::Result<std::string::String, anyhow::Error> {
        let t = template::Template::from_str(&self.parser_config, input)?; // TODO: This is a weird API. Possibly make a Parser struct, held by Trebuchet, that holds the ParserConfig inside
        let output = t.process()?; // TODO: Make the engine be the one processing
        Ok(output)
    }
}
