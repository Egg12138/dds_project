
sys = require("sys")
-- LUATOS SoC will recursively scan all files under the directory
local rtchelper = require("rtchelper")
local httphelper = require("http-utils")
local wifi = require("wifi-manager")
local checker = {}

function checker.rtc_checks()
  rtchelper.display_time()
  rtchelper.set_time(2014,11,14)
end
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

function checker.http_checks()
  HTTPBIN = "http://httpbin.io/"
  local ret_code = httphelper.get(HTTPBIN.."ip") 
  if ret_code == 0 then
    log.info("===CHECK::HTTP", "request works")
  else 
    log.info("===CHECK::HTTP", "error:" .. ret_code)
  end
  httphelper.get(HTTPBIN.."get?foo=bar")
  httphelper.get(HTTPBIN.."user-agent")
  httphelper.get(HTTPBIN.."dump/request?foo=bar")
  httphelper.get(HTTPBIN.."status/418")
  httphelper.get(HTTPBIN.."status/418")

function checker.mqtt_tests()
  
end



end

return checker