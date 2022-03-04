local M = {}

function M.print_rule(rule) 
  return _print_rule(rule)
end

function M.create_default_rule()
  return _templar_create_default_rule()
end

return M
