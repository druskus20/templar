use super::config::Rule;
use anyhow::Result;
use rlua;
use rlua::prelude::*;
use rlua::ExternalResult;

// define a function in the lua context
macro_rules! set_lua_functions {
    (
        $lua_context:expr, $(
            $name:literal = $value:expr
        ),*
    )
    => {
        $(
            let function = $lua_context.create_function($value)?;
            let context =  $lua_context.globals();
            context.set($name, function)?;
        )*
    }
}

pub fn register_lua_api(lua: &Lua) -> Result<()> {
    lua.context(|lua_context| {
        set_lua_functions!(
            lua_context,
            "print_rule" = |_, x| print_rule(x).to_lua_err(),
            "_create_default_rule" = |_, ()| _create_default_rule().to_lua_err()
        );
        LuaResult::Ok(())
    })?;
    Ok(())
}

fn print_rule(lua_rule: Rule) -> Result<()> {
    dbg!(lua_rule);
    Ok(())
}

fn _create_default_rule() -> Result<Rule> {
    Ok(Rule::default())
}
