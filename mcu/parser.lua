utils = require("utils")
local PARSER = {}
local logs = require("log_context").zhlog


local function cmdtype(cmdtable)
  assert(type(decoded) == "table", logs.not_table)
  assert(decoded.command_name, logs.invalid_command)

  return decoded.command_name
end

local function paras(cmdtable)
  assert(type(decoded) == "table", logs.not_table)
  return cmdtable.paras
end

function PARSER.type_and_paras(cmdtable)
  local newt = {}
  newt.command_name = cmdtype(cmdtable)
  newt.paras = paras(cmdtable)
  return newt.command_name, newt.paras
end

function PARSER.json2table(str)
  local decoded = json.decode(str)
  return decoded
end

return PARSER