local templar = {}

templar.print_rule = function(rule)
  templar_print_rule(rule)
end

templar.create_default_rule = function ()
  _templar_create_default_rule()
end

return templar
