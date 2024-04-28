PROJECT = "DDS-Demo"
VERSION = "0.1.7"

_G.sys = require("sys")
require("sysplus")

--[==[
TODO: 移动到 connection, utils 等目录下
--]==]

checker = require("checks")
wifi = require("wifi-manager")
mqtthelper = require("mqtts")
handler = require("data_handler")
DDS = require("ddss")
MCU = require("mcus")

notice = require("log_context") -- Chinese Logs
logs = notice.zhlog

DATA_STREAM = ""
COMMUNICATION_MODE = "MQTT" -- MQTT, Socket, ...


function esp32c3_init() 
  -- setup to **OUTPUT** mode, `setup` returns closure to set(output)/get(input) the level
  CS = gpio.setup(MCU.C3.SS, OUTPUT)
  UPD = gpio.setup(MCU.C3.UPD, OUTPUT)
  RST = gpio.setup(MCU.C3.RST_DDS, OUTPUT)
  SYNC = gpio.setup(MCU.C3.SYNC, OUTPUT)
  INTR = gpio.setup(MCU.C3.INTR, INPUT, PULLUP) -- 内部 PULLUP

end

function esp32s3_init()
  CS = gpio.setup(MCU.S3.SS, OUTPUT)
  UPD = gpio.setup(MCU.S3.UPD, OUTPUT)
  RST = gpio.setup(MCU.S3.RST_DDS, OUTPUT)
  SYNC = gpio.setup(MCU.S3.SYNC, OUTPUT)
  INTR = gpio.setup(MCU.S3.INTR, INPUT, PULLUP) -- 内部
end

function mcu_init(board)
  if board == "esp32c3" or board == "ESP32C3" then
    esp32c3_init()
  elseif board == "esp32s3" or board == "ESP32S3" then
    esp32s3_init()
  else
    log.error("MCU-INIT", logs.unsupported_mcu ,board)
  end
end



function init_system() 

  wifi.simpleRun()
  mcu_init("esp32c3")

  handler.setup_spi()
  handler.init_DDS()
  
end



sys.taskInit(function()

  init_system()
  log.info("Main", "Started...")
  sys.wait(1000)

  if COMMUNICATION_MODE == "Socket" then
    log.info("Communcation", logs.communicate_socket, logs.unsupported)
  elseif COMMUNICATION_MODE == "IoT" then
    log.info("Communcation", logs.communicate_iot)
    --[==[
     TODO: mqtt runner 
     循环：
      等待MQTT:
      接收MQTT:
      响应MQTT:
    --]==]

  end
end)

-- sys.subscribe("WIFISTART", function()
--   wlan.powerSave(wlan.PS_NONE)
-- end)

-- sys.subscribe("WIFIPAUSE", function()
--   log.info("wlan power save", "pause. waiting for new commands")
--   wlan.powerSave(wlan.PS_MAX_MODEN)
-- end
-- )

sys.run()