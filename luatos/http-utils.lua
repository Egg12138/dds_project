_G.sys = require "sys"
require "sysplus"
local http_helper = {}

-- CODE_TYPES = {
--   Message = 0,
--   Success = 1,
--   Redirect = 2,
--   RequestErr = 3,
--   ServerErr = 4,
-- }

-- function kinds_ofcode(code)
--   local msg_codes = ranges(100, 200)
--   local success_codes = ranges(200, 300)
--   local rediret_code = 300
--   local reqerr_codes = ranges(300, 400)
--   if success_codes[code] then
--     return CODE_TYPES.Success
--   elseif reqerr_codes[code] then
--     return CODE_TYPES.RequestErr
--   elseif msg_codes[code] then
--     return CODE_TYPES.Message
--   elseif redirect_codes[code] then
--     return CODE_TYPES.Redirect
--   else
--     return CODE_TYPES.ServerErr
--   end

-- end

-- http_helper.get { url, timeout = 120000} -- the default timeout is 12s

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