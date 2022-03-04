local M = {}

function M.foo()
	return foo()
end

function M.bar(_argum)
	return bar(_argum)
end

return M
