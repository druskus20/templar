use super::rule::Rule;
use super::TemplarConfig;
use anyhow::Result;
use lua_export::*;

pub(crate) use lua_functions::gen_lua_wrapper;
pub(crate) use lua_functions::register_lua_api;

#[lua_export_mod]
mod lua_functions {
    /*
     * NOTE: Every function must take a config as the first parameter at the moment
     */
    use std::sync::Arc;
    use std::sync::Mutex;

    use super::*;

    #[lua_export]
    fn print_rule(config: Arc<Mutex<TemplarConfig>>, lua_rule: Rule) -> Result<()> {
        println!("{:?}", lua_rule);
        dbg!(config);
        Ok(())
    }

    #[lua_export]
    fn _create_default_rule(config: Arc<Mutex<TemplarConfig>>) -> Result<Rule> {
        dbg!(config);
        Ok(Rule::default())
    }

    #[lua_export]
    fn setup(config: Arc<Mutex<TemplarConfig>>) -> Result<()> {
        dbg!(config);
        Ok(())
    }

    #[lua_export]
    fn print_config(config: Arc<Mutex<TemplarConfig>>) -> Result<()> {
        println!("{:?}", config);
        dbg!(config);
        Ok(())
    }

    #[lua_export]
    fn add_rule_to_config(config: Arc<Mutex<TemplarConfig>>, rule: Rule) -> Result<()> {
        config.lock().unwrap().rules.push(rule); // unwrap?
        Ok(())
    }
}
