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

print("Default rule ----\n")
local default_rule = _create_default_rule()
print(dump(default_rule))
print_rule(default_rule)

print("\n\nRule from lua ----\n")
local rule_from_lua = {
   id = "rule_from_lua",
   targets = "*",
   basepath = "./",
   rules = {},
}
print(dump(rule_from_lua))
print_rule(rule_from_lua)

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
print_rule(rule_with_nested_rules)
