
sys = require("sys")
-- LUATOS SoC will recursively scan all files under the directory
local wifi = require("wifi-manager")
local checker = {}

function checker.mem_check()
  log("luatos")
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


-- [[
--  @description: rename from cpu_check to machine check
-- ]]
function checker.machine_check()
  -- TODO
  checker.rtc_checks()
  checker.mem_check()
  checker.wifi_checks()
  checker.bluetooth_checks()

end

function checker.bluetooth_checks()
end

function checker.bootloader_check()
  -- LEARN: bootloader infomation parsing
end



function checker.wifi_checks()
  wifi.init()
end


function checker.bt_check()
  -- LEARN bluetooth configuration check
  -- LEARN bluetooth status check 
end

return checker