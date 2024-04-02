_G.sys = require("sys")
TOAP_SSID = "PROV_eggs"
TOAP_PWD = "liyuan11328"

SCREEN_PLACEHOLDER = {} -- 

function SCREEN_PLACEHOLDER.init()
  log.info("Screen PlaceHolder", "init...")
end

function SCREEN_PLACEHOLDER.get_pwd()
  return TOAP_PWD
end


function SCREEN_PLACEHOLDER.get_ssid()
  -- repl()
  -- getdata() ...
  return TOAP_SSID
end

function SCREEN_PLACEHOLDER.display(msg)
end

return SCREEN_PLACEHOLDER