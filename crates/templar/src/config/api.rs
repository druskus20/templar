use super::rule::Rule;
use super::TemplarConfig;
use anyhow::Result;
use lua_export::*;
use rlua;

/*
 * TODO: I need some sort of global mutable state that I can modify with the api calls
 * (eww tho)
 */

// Exports functions defined in the macro
pub use lua_functions::*;

#[lua_export_mod]
pub(super) mod lua_functions {
    use super::*;

    #[lua_export]
    fn print_rule(lua_rule: Rule) -> Result<()> {
        println!("{:?}", lua_rule);
        Ok(())
    }

    #[lua_export]
    fn _create_default_rule() -> Result<Rule> {
        Ok(Rule::default())
    }

    #[lua_export]
    fn setup(config: TemplarConfig) -> Result<TemplarConfig> {
        Ok(config)
    }

    #[lua_export]
    fn print_config(config: TemplarConfig) -> Result<()> {
        println!("{:?}", config);
        Ok(())
    }
}
