_G.sys = require "sys"
require "sysplus"
local http_helper = {}

function http_helper.get(url, timeout)
  -- setmetatable(url, { __index = {timeout = 120000}})
  timeout = timeout or 120000
  sys.wait(100)
  log.info("HTTP-C", "request starts...")
  local http_code, headers, body = 
    http.request("GET", url).wait(timeout)
  log.info("HTTP-GET", http_code , headers)
  if http_code > 100 then
    log.info("HTTP-GET", "are head data: ".. body)
  else 
    log.info("HTTP-GET", "not head data" .. body)
  end

  -- local kind = kinds_ofcode(http_code)
  

  if http_code >= 200 and http_code < 300 then
    return 0
  else 
    return http_code
  end
end


  


return http_helper