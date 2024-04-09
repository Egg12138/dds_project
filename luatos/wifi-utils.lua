_G.sys = require("sys")

ESP32APSSID = "PROV_EGG"
ESP32APPWD = "12345678"


local wifi_helper = {}

local scan_result = {}

SCREEN_PLACEHOLDER = require "screen0"




function wifi_helper.fetch_ssid_info()
  local ssid = SCREEN_PLACEHOLDER.get_ssid()
  local pwd = SCREEN_PLACEHOLDER.get_pwd()
  local info = {}
  info.ssid = ssid
  info.pwd = pwd
  info.mode = "STATION"
  return info
end

function wifi_helper.setup(isdebug)
  local info = wifi_helper.fetch_ssid_info()
  wifi_setup(info, isdebug)
end


-- default to be at MODE: STATION
function wifi_setup(info, debug)
  local mode = info.mode
  local ssid =info.ssid
  local pwd = info.pwd
  debug = debug or false
  sys.wait(100)
  wlan.hostname("DDS-esp32c3")
  wlan.init()
  wlan.setMode(mode)
  sys.wait(100)
  if mode == wlan.AP then
    wlan.createAP(ESP32APSSID .. wlan.getMac(), ESP32APPWD )
    log.info("AP", ESP32APSSID .. wlan.getMac(), ESP32APPWD)

  elseif mode == wlan.STATION then
    wlan.connect(ssid,pwd, 1)
    if wlan.ready() then
      SCREEN_PLACEHOLDER.display("IP: ")
      log.info("STATION", "is ready, IP: " .. wlan.getIP())
    end
      SCREEN_PLACEHOLDER.display("IP: ")
    log.info("STATION", ret)
  end

  if debug then
    wifi_helper["MAC"] = wlan.getMac(0, true) 
    wifi_helper["IPv4"] = wlan.getIP()
    wifi_helper["APinfo"] = wlan.getInfo()
  end

end

function wifi_helper.wifi_test()
  wlan.scan()
  httpsrv.start(80, function(fd, method, uri, headers, body)
    log.info("httpsrv", method, uri, json.encode(headers), body)
    -- /led是控制灯的API
    -- 扫描AP
    if uri == "/scan/go" then
        wlan.scan()
        return 200, {}, "ok"
    -- 前端获取AP列表
    elseif uri == "/scan/list" then
        return 200, {["Content-Type"]="applaction/json"}, (json.encode({data=_G.scan_result, ok=true}))
    -- 前端填好了ssid和密码, 那就连接吧
    elseif uri == "/connect" then
        if method == "POST" and body and #body > 2 then
            local jdata = json.decode(body)
            if jdata and jdata.ssid then
                -- 开启一个定时器联网, 否则这个情况可能会联网完成后才执行完
                sys.timerStart(wlan.connect, 500, jdata.ssid, jdata.passwd)
                return 200, {}, "ok"
            end
        end
        return 400, {}, "ok"
    -- 根据ip地址来判断是否已经连接成功
    elseif uri == "/connok" then
        return 200, {["Content-Type"]="applaction/json"}, json.encode({ip=socket.localIP()})
    end
    -- 其他情况就是找不到了
    return 404, {}, "Not Found" .. uri
  end)

  log.info("web", "pls open url http://192.168.4.1/")

  wifi_helper.infos()

end


function wifi_helper.infos()
  if wlan.ready() then
    log.info("WifiInfo:", "AP: " .. json.encode(wifi_helper["APinfo"]))
    log.info("WifiInfo:", "AP: " , wifi_helper["APinfo"])
    log.info("WifiInfo:", "IPv4:" .. wifi_helper["IPv4"])
    log.info("WifiInfo:", "MAC:" ..wifi_helper["MAC"])
  else 
    log.info("Wifi:", "not ready")
  end
end

return wifi_helper