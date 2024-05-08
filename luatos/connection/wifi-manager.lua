_G.sys = require("sys")



-- WiFi管理器，用于存储和管理WiFi网络配置
HOST_NAME = "DDS-esp32c3" -- 设备主机名
SSID = "PROV_eggs" -- 内置SSID
PWD = "liyuan11328" -- 内置密码
SSID_AS_AP = "esp32c3-ap577" -- 设备作为AP的SSID
PWD_AS_AP = "12345678" -- 设备作为AP的密码

local manager = {} -- WiFi管理器的实例

manager.wifilist = {} -- 存储已知WiFi网络列表
manager.ssidlist = {} -- 存储SSID列表
manager.wifinums = 0  -- 已知WiFi网络数量

-- 初始化fskv（一个键值存储库）
if not fskv.init() then
  log.error("fskv", "init failed")
end

-- 打印表信息的调试函数
function manager.dbg(t)
  for k,v in pairs(t) do
    log.info("\t\tdbg", k, v)
  end
end

-- 保存WiFi网络列表到fskv
function manager.save()
  for _, value in ipairs(manager.wifilist) do
    if value.ssid and value.pwd then
      fskv.sett("wifilist", value.ssid, value.pwd)
    end
  end
end

-- 从fskv加载WiFi网络列表
function manager.load()
  tmp = fskv.get("wifilist")
  if tmp == nil then
    return
  end
  for ssid, pwd in pairs(tmp) do
    log.info("wifilist loader", ssid, pwd)
    if not manager.has(ssid) then
      manager.add(ssid, pwd)
    end
  end
end

-- 从fskv中移除指定SSID的WiFi配置
function manager.remove_fromfskv(ssid)
  fskv.sett("wifilist", ssid)
end

-- 添加一个新的WiFi配置到列表中
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

-- 根据SSID获取密码
function manager.getpwd(ssid)
  for _, wifiob in pairs(manager.wifilist) do
    if wifiob.ssid == ssid then
      return wifiob.pwd
    end
  end 
end

-- 检查是否已存在指定SSID的WiFi配置
function manager.has(ssid)
  return manager.getpwd(ssid) ~= nil
end

-- 检查指定SSID的WiFi是否已被发现
function manager.isdiscoer(ssid)
  local wifiobj = manager.get(ssid)
  if wifiobj then
    return wifiobj.discovered
  end
end

-- 标记指定SSID的WiFi为已发现
function manager.found(ssid)
  local wifiobj = manager.get(ssid)
  if wifiobj then
    wifiobj.discovered = true
  end
end

-- 在Station模式或AP模式下进行WiFi扫描和连接
function manager.scan_and_connect(mode,timeout)
  timeout = timeout or 1200
  sys.wait(100)
  wlan.setMode(mode)

  if mode == wlan.STATION then
    manager.connect()
    while not wlan.ready() do
      sys.wait(timeout)
      manager.connect()
      local ret, ip = sys.waitUntil("IP_READY",1000)
      if ret or ip then
        break
      end
      log.info("wlan", "STA MAC".. wlan.getMac(), "try to reconnect")
      sys.wait(100)
    end

      log.info("wlan", "IP_ready")
      log.info("ip", ip)
      if ip then
        _G.ip = ip
      end

  elseif mode == wlan.AP then
    wlan.createAP(SSID_AS_AP .. wlan.getMac())
    log.info("AP", PWD_AS_AP .. wlan.getMac())
  else
    log.error("wifi manager", "illegal mode", mode)
  end
end

-- 初始化WiFi模块并添加默认WiFi配置
function manager.init()
  manager.add(SSID, PWD)
  wlan.hostname(HOST_NAME)
  wlan.init()
end

-- 执行WiFi扫描
function manager.scan()
  wlan.scan()
end

-- 返回WiFi配置列表的迭代器
function manager.pairs()
  return pairs(manager.wifilist)
end

-- 设置最后连接的WiFi配置
function fskv_seT_last(ssid,pwd)
  fskv.set("last_wlan_ssid", ssid)
  fskv.set("last_wlan_pwd", pwd)
end

-- 尝试连接到指定的WiFi网络
function manager.connect()
  if wlan.ready() then
    return
  end

  if fskv.kv_get("last_wlan_ssid") then
    wlan.init()
    wlan.setMode(wlan.STATION)
    wlan.connect(fskv.kv_get("last_wlan_ssid"), fskv.kv_get("last_wlan_pwd"))
    if wlan.ready() then
      log.info("fskv", "quickly connected to last one")
      return
    end
  end

  for _,ssid in pairs(manager.ssidlist) do
    wlan.connect(ssid, manager.getpwd(ssid))
    if wlan.ready() then
      fskv_seT_last(wifiobj.ssid, wifiobj.pwd)
      return
    end
  end


  wlan.connect(SSID, PWD)


end

-- 断开WiFi连接
function manager.disconnect()
  wlan.disconnect()
  log.info("wifi manager", "disconnected")
end

-- 简单运行模式：初始化、加载配置、扫描并尝试连接
function manager.simple_run()
  manager.init()
  manager.load()
  manager.scan_and_connect(wlan.STATION, 1000)
  log.info("wifimanager", "Ok. ")
  wlan.powerSave(wlan.PS_MIN_MODEM)
  log.info("wifimanager", "Wlan Powermode is set to PS_MIN_MODEM ")
end

return manager