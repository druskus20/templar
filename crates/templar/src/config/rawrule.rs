use std::collections::HashMap;

use rlua::prelude::{FromLua, LuaContext, LuaValue, ToLua};

use crate::hashmap;

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub(crate) struct RawRule {
    pub id: String,
    pub targets: String,
    pub rules: Vec<RawRule>,
    pub basepath: String,
}

impl<'lua> FromLua<'lua> for RawRule {
    fn from_lua(lua_value: rlua::Value<'lua>, _: rlua::Context<'lua>) -> rlua::Result<Self> {
        if let LuaValue::Table(lua_table) = lua_value {
            Ok(RawRule {
                id: lua_table.get("id")?,
                targets: lua_table.get("targets")?,
                rules: lua_table.get("rules")?,
                basepath: lua_table.get("basepath")?,
            })
        } else {
            Err(rlua::Error::FromLuaConversionError {
                to: "Rule",
                from: "LuaValue",
                message: Some("Expected rule to be a lua table".to_string()),
            })
        }
    }
}

impl<'lua> ToLua<'lua> for RawRule {
    fn to_lua(self, lua: rlua::Context<'lua>) -> rlua::Result<LuaValue<'lua>> {
        // TODO: Maybe figure out a way that doesnt require creating a HashMap first..?
        let hashmap: HashMap<&str, LuaValue> = hashmap!(
            "id" => self.id.to_lua(lua)?,
            "targets" => self.targets.to_lua(lua)?,
            "rules" => self.rules.to_lua(lua)?,
            "basepath" => self.basepath.to_lua(lua)?,
        );
        Ok(LuaValue::Table(LuaContext::create_table_from(
            lua, hashmap,
        )?))
    }
}
