sys = require("sys")
local checker = {}
function checker.mem_check()
  log("lua o")
  sys.wait(1000)
  log.info("luatos", "hi", count, os.date())
  local tot, used, history_max_used = rtos.meminfo()
  log.info("lua VM", "tot: "..tot .. "used: "..used .. 
    "historical max used:" ..history_max_used)
  tot, used, history_max_used = rtos.meminfo("sys")
  log.info("sys", "tot: "..tot .. "used: "..used .. 
    "historical max used:" ..history_max_used)
  tot, used, history_max_used = rtos.meminfo("psram")
  log.info("psram", "tot: "..tot .. "used: "..used .. 
    "historical max used:" ..history_max_used)
end


function checker.cpu_check()
  -- TODO
end


function checker.bootloader_check()
  -- LEARN: bootloader infomation parsing
end



function checker.wifi_check()
  -- LEARN wifi configuration check
  -- LEARN wifi status check 

end


function checker.bt_check()
  -- LEARN bluetooth configuration check
  -- LEARN bluetooth status check 
end


return checker