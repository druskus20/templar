use super::rule::Rule;
use anyhow::Result;
use lua_export::*;
use rlua;

// Exports functions defined in the macro
pub use lua_functions::*;

#[lua_export_mod]
pub(super) mod lua_functions {
    use super::*;

    #[lua_export]
    fn print_rule(lua_rule: Rule) -> Result<()> {
        dbg!(lua_rule);
        Ok(())
    }

    #[lua_export]
    fn _create_default_rule() -> Result<Rule> {
        dbg!(Rule::default());
        Ok(Rule::default())
    }
}
