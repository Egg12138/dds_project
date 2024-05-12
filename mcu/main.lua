PROJECT = "DDS-Demo"
VERSION = "0.1.9"

_G.sys = require("sys")
require("sysplus")

--[==[TODO: 移动到 connection, utils 等目录下
--]==]


wifi = require("wifi-manager")
iot = require("mqtts")
handler = require("data_handler")
MCU = require("mcus")

notice = require("log_context") -- Chinese Logs
logs = notice.zhlog

DATA_STREAM = ""
COMMUNICATION_MODE = "IoT" -- MQTT, Socket, ...



function esp32c3_init() 
  -- setup to **OUTPUT** mode, `setup` returns closure to set(output)/get(input) the level
  CS = gpio.setup(MCU.C3.CS, OUTPUT)
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

  wifi.simple_run()
  log.info("before init:debug", (CS ~= nil), (UPD ~= nil), (RST ~= nil), (SYNC ~= nil), (INTR ~= nil))
  mcu_init("esp32c3")
  log.info("after init:debug", (CS ~= nil), (UPD ~= nil), (RST ~= nil), (SYNC ~= nil), (INTR ~= nil))
  handler.setup_spi()
  handler.init_DDS()
  
end



function mqtt_runner()

  while not wlan.ready() do
    log.info("MQTT runner", "wifi", logs.notready)
    wifi.simple_rerun()
  end

iot_clientid, iot_user, iot_pwd = iotauth.iotda(
    iot.device_id,
    iot.device_secret
  )

  log.info("IoTDA", iot_clientid, iot_user, iot_pwd)

  mqttc = mqtt.create(nil, iot.iot_url, iot.port)

  mqttc:auth(iot_clientid, iot_user, iot_pwd)
  mqttc:keepalive(30)
  mqttc:autoreconn(true, 3000)
  log.info("MQTT", logs.iot_authorize_ok)
  mqttc:on(function(mqtt_client, event, topic, payload, metas)
    log.info("MQTT", "event happened", event, mqtt_client)
    log.info("MQTT:debug", "event topic", topic, "payload", payload, "table ID", metas)
    if event == "conack" then
      -- TODO: 报文格式解析
      sys.publish("MQTT-conack")
      mqttc:subscribe(iot.topics.cmds_dSpP, 1)
    elseif event == "sent" then
      log.info("MQTT:sent", "topic", topic, payload, logs.fin)
      sys.publish("MQTT-sent", payload, topic)
    elseif event == "recv" then
      --- 队列阻塞的情况
      sys.publish("MQTT-receive", topic, payload)
    else
      log.info("MQTT", logs.invalid_command)
    end
  end)

  mqttc:connect()

  mqttc:publish(iot.topics.msg_up_dPpS,"原神启动",1)
end


sys.taskInit(function()

  init_system()
  -- wifi.simple_run()
  log.info("Main", "MQTT communication started...")
  sys.wait(1000)
  mqtt_runner()
  sys.waitUntil("disconnect")
  mqttc:close()
  mqttc = nil

end)

sys.subscribe("MQTT-conack", function()
  log.info("MQTT:Conack", logs.iot_conack)
  log.info("DDSController", logs.usage)
end
)

sys.subscribe("MQTT-receive", function(topic, data)
  log.info("MQTT:receive", topic, logs.received_datapkg, data)
  handler.handle_received(data)
  log.info("MQTT:receive", logs.fin)
end
)

sys.timerLoopStart(function()
  collectgarbage("collect")
end
, 5000)
sys.run()