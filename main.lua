PROJECT = "DDS-DEMO"
VERSION = "0.1.0"

_G.sys = require("sys")
checker = require("./checks")

-- TODO: finished this...
sys.taskInit(function ()
  
  -- esp-idf : cpu_info 
  -- esp-idf: wifi config status
  -- esp-idf: bluetooth config, status
  -- meminfo
  -- wifi init
  -- bluetooth init 
  -- RAINMAKER
  -- coroutine:?
    -- monitor the input callback data 
    -- wait for events from { RAINMAKER, COMMANDLINE}
  
end )
sys.run()