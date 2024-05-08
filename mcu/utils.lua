local KITS = {}
sys = require("sys")

-- @param:root table的根节点
-- show all fields of a table, recursively
function KITS.all_fields_rec(root)
  local cache = { [root] = "." }
  local function _dump(t,space,name)
      local temp = {}
      for k,v in pairs(t) do
          local key = tostring(k)
          if cache[v] then
              table.insert(temp,"*=" .. key .. " {" .. cache[v].."}")
          elseif type(v) == "table" then
              local new_key = name .. "." .. key
              cache[v] = new_key
              table.insert(temp,"*=" .. key .. _dump(v,space .. (next(t,k) and "|" or " " )..                                             string.rep(" ",#key),new_key))
          else
              table.insert(temp,"*=" .. key .. " [" .. tostring(v).."]")
          end
      end
      return table.concat(temp,"\n"..space)
  end
  print(_dump(root, "" , ""))
end


function KITS.all_fields(t)
  log.info("debug", "check table", t)
  for k,v in pairs(t) do
    local key = tostring(k)
    print("* "..key.." [" .. tostring(v) .."]")
  end
end

function KITS.contains(t,pattern) 
  for k,v in pairs(t) do
    if v == pattern then
      return true
    end
  end
  return false
end


return KITS