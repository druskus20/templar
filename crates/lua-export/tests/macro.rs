#[macro_use]
extern crate lua_export;

#[lua_export_mod(path = "./crates/lua-export/tests/macro.lua")]
mod lua_export_test {

    #[lua_export]
    pub fn foo() -> String {
        "foo".to_string()
    }

    #[lua_export]
    pub fn bar(_argum: String) -> String {
        "bar".to_string()
    }

    #[cfg(test)]
    mod test {
        #[test]
        fn test_lua_export() {
            dbg!(super::LUA_FUNCTIONS);
            assert_eq!(super::foo(), "foo");
            assert_eq!(super::bar("woo".to_string()), "bar");
        }
    }
}
