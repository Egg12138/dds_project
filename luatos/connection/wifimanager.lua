_G.sys = require("sys")

-- PROJECT = "DDS-esp32c3-refactor"
-- VERSION = "0.1.4"

HOST_NAME = "DDS-esp32c3"
BUILTIN_SSID = "PROV_eggs"
BUILTIN_PWD = "liyuan11328"

SSID_AS_AP = "esp32c3-ap577"
PWD_AS_AP = "12345678"


local manager = {}

manager.wifilist = {}

manager.ssidlist = {}

manager.wifinums = 0  

-- manager.had_connected = false -- 本次上电后时候连接过，若true，则可以connect()直接重连

if not fskv.init() then
  log.error("fskv", "init failed")
end

function dbg(t)
  for k,v in pairs(t) do
    log.info("\t\tdbg", k, v)
  end
end
-- remained for main.lua
-- BTN_BOOT = 9 -- NOTICE, 清除配网信息按钮
-- gpio.debounce(BTN_BOOT, 1000)
-- gpio.getup(BTN_BOOT, function()
--   log.info(
--     "GPIO boot", 
--     "gpio boot button reseted")
--     sys.publish("BTN_BOOT")
-- end
-- )


-- save to fskv
function manager.save()
  for _, value in ipairs(manager.wifilist) do
    log.info("wifilist saver: save", value.ssid)
    if value.ssid and value.pwd then
      fskv.sett(
        "wifilist",
        value.ssid, value.pwd)
    end
  end
end

function manager.load()
  tmp = fskv.get("wifilist")
  log.info("fskv", fskv.get("wifilist"), json.encode(fskv.get("wifilist")))
  if tmp == nil then
    return
  end
    -- travel tmp: ssid,pwd, if not manager.wifilist.has(ssid), append it to wifilist
  for ssid, pwd in pairs(tmp) do
    log.info("wifilist loader", ssid, pwd)
    if not manager.has(ssid) then
      manager.add(ssid, pwd)
    end
  end

  for _, wifiob in pairs(manager.wifilist) do
    table.insert(manager.ssidlist, wifiob.ssid)
  end
end

function manager.remove_fromfskv(ssid)
  fskv.sett("wifilist", ssid)
end


-- [[
-- manager.wifilist.__fields:
-- {1. ssid, 2. pwd, 3. discovered}
--
-- ]]
function manager.add(ssid, pwd)
  local tmp = ssid:trim():split(".")
  if #tmp > 1 then
    log.error("wifi manager", "illegal SSID: should not contains `.`", ssid)
  end
  
  wifiobj = {
    ssid = ssid,
    pwd = pwd,
    discovered = false,
  }

  table.insert(manager.wifilist, wifiobj)
  manager.wifinums = manager.wifinums + 1

end

function manager.getpwd(ssid)
  for _, wifiob in pairs(manager.wifilist) do
    if wifiob.ssid == ssid then
      return wifiob.pwd
    end
  end 
end

-- return false if manager.get(ssid) returns nil
-- return pwd if manager.get(ssid) returns something
function manager.has(ssid)
  return manager.getpwd(ssid) ~= nil
end

function manager.isdiscoer(ssid)
  local wifiobj = manager.get(ssid)
  if wifiobj then
    return wifiobj.discovered
  end
end

function manager.found(ssid)
  local wifiobj = manager.get(ssid)
  if wifiobj then
    wifiobj.discovered = true
  end
end

function manager.scan_and_connect(mode,timeout)
  timeout = timeout or 1200 -- ms
  sys.wait(100)
  wlan.setMode(mode)

  if mode == wlan.STATION then
    
    manager.connect()

    while not wlan.ready() do
      sys.wait(timeout) -- retry time (ms)
      manager.connect()
      local ret, ip = sys.waitUntil("IP_READY",30000)
      log.info("wlan", "finnaly ready")
      log.info("ip", ip)
      if ip then
        _G.ip = ip
      end
      log.info("wlan", "STA MAC".. wlan.getMac())
      sys.wait(100)
    end

  elseif mode == wlan.AP then
    wlan.createAP(SSID_AS_AP .. wlan.getMac())
    log.info("AP", PWD_AS_AP .. wlan.getMac())
  else
    log.error("wifi manager", "illegal mode", mode)
  end

end


function manager.init()
  manager.add(BUILTIN_SSID, BUILTIN_PWD)
  wlan.hostname(HOST_NAME)
  wlan.init()
end

function manager.scan()
  wlan.scan()
end

-- return (_, wifiobj)
function manager.pairs()
  return pairs(manager.wifilist)
end

function fskv_seT_last(ssid,pwd)
  fskv.set("last_wlan_ssid", ssid)
  fskv.set("last_wlan_pwd", pwd)
end
-- TODO small bugs, reduntant logic.
function manager.connect()
  
  if wlan.ready() then
    return
  end


  if fskv.kv_get("last_wlan_ssid") then
    wlan.connect(fskv.kv_get("last_wlan_ssid"), fskv.kv_get("last_wlan_pwd"))

    if wlan.ready() then
      log.info("wifi manager", "connect to the most recent connection")
    return
    end
  end

  for _,ssid in pairs(manager.ssidlist) do
      wlan.connect(ssid, manager.getpwd(ssid))
      log.info("wifi manager", "connecting to", ssid, manager.getpwd(ssid))
      if wlan.ready() then
        log.info("wifi manager", "connected to", ssid)
        fskv_seT_last(wifiobj.ssid, wifiobj.pwd)
        return
      end
  end
end

function manager.disconnect()
  wlan.disconnect()
  log.info("wifi manager", "disconnected")
end


-- sys.taskInit(function()
--   log.setLevel(2)
--   manager.init()
--   manager.load()
--   manager.scan_and_connect(wlan.STATION)
--   manager.save()
--   wlan.powerSave(wlan.PS_MAX_MODEM)
--   log.info("wifimanager", "OK.")
-- end)

function manager.simpleRun()
  manager.init()
  manager.load()
  manager.scan_and_connect(wlan.STATION, 1000)
  log.info("wifimanager", "Ok. ")
  wlan.powerSave(wlan.PS_MIN_MODEM)
  log.info("wifimanager", "Wlan Powermode is set to PS_MIN_MODEM ")
end

return manager


