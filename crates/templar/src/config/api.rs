use super::config::Rule;
use anyhow::Result;
use lua_export::*;
use rlua;

pub use lua_functions::gen_lua_wrapper;
pub use lua_functions::register_lua_api;

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
