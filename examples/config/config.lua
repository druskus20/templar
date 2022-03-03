local templar = require("templar")

local dump = function(o)
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

local r = _templar_create_default_rule()
templar_print_rule(r)

print("Default rule ----\n")
local default_rule = templar.create_default_rule()
print(dump(default_rule))
templar.print_rule(default_rule)

print("\n\nRule from lua ----\n")
local rule_from_lua = {
   id = "rule_from_lua",
   targets = "*",
   basepath = "./",
   rules = {},
}
print(dump(rule_from_lua))
templar.print_rule(rule_from_lua)

print("\n\nRule with nested rules ----\n")
local rule_with_nested_rules = {
   id = "rule_with_nested_rules",
   targets = "*",
   basepath = "./",
   rules = {
      {
         id = "nested_rule",
         targets = "*",
         basepath = "./",
         rules = {},
      },
   },
}
print(dump(rule_with_nested_rules))
templar.print_rule(rule_with_nested_rules)
