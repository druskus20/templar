function dump(o)
   if type(o) == 'table' then
      local s = '{ '
      for k,v in pairs(o) do
         if type(k) ~= 'number' then k = '"'..k..'"' end
         s = s .. '['..k..'] = ' .. dump(v) .. ','
      end
      return s .. '} '
   else
      return tostring(o)
   end
end

local templar = require("example")

print("Default rule ----\n")
local default_rule = templar._create_default_rule()
templar.print_rule(default_rule)

print("\n\nRule from lua ----\n")
local rule_from_lua = {
   id = "rule_from_lua",
   targets = "*",
   basepath = "./",
   rules = {},
}
templar.print_rule(rule_from_lua)

print("\n\nRule with nested rules ----\n")
local rule_with_nested_rules = {
   id = "rule_with_nested_rules",
   targets = "templar.lua",
   basepath = "./",
   rules = {
      {
         id = "nested_rule",
         targets = "config.lua",
         basepath = "./",
         rules = {},
      },
   },
}

templar.print_rule(rule_with_nested_rules)

local config = {
   rules = {
      default_rule,
      rule_from_lua,
      rule_with_nested_rules,
   },
   dest_base = "/home/druskus/",
}

templar.setup(config)

print("\n\nConfig ----\n")
templar.print_config(config)

