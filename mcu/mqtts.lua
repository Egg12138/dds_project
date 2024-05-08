require("sysplus")
require("data_handler")
local sys = require "sys"
datapkg = {
  command_name = "",
  paras = "0x123124",
  request_it = "2",
}


local MQTT = {}

MQTT.iot_url = "9ceb993f1d.st1.iotda-device.cn-south-1.myhuaweicloud.com"
MQTT.device_id = "660d43201b5757626c1b700f_0403demo"
MQTT.device_secret = "liyuan11328"    --设备密钥
MQTT.port = 1883

MQTT.topics = {}
-- d{P, S}p{P, S} => device: Publisher or Subscriber; platform: Publisher or Subscriber
MQTT.topics.msg_up_dPpS = "$oc/devices/"..MQTT.device_id.."/sys/messages/up"
MQTT.topics.cmds_dSpP = "$oc/devices/"..MQTT.device_id.."/sys/commands"

local mqttc = nil

function MQTT.cmd_handler(request)

        cmd = request.command_name
        paras = request.paras
        if cmd == "input" then
            log.info("CmdHandler", "set input")
            freq,volt,phase = paras.freq_hz, paras.volt_mv, paras.phs_oft
            log.info("CmdHandler", "parsed",
            "frequency(Hz)", freq, 
            "voltage(mV)", volt,
            "phase(degree)", phase)
        elseif cmd == "spi" then
          log.info("CmdHandler", "spi cmds", paras)  
          -- one line command
          spi_cmd = paras
          spi_cmds_transfer(spi_cmd)

        elseif cmd == "report" then 
        elseif cmd == "poweron" then
        elseif cmd == "poweroff" then
        elseif cmd == "scan" then
        end
end
function handle(payload)

    data, err = json.decode(payload)

    if err then
        log.info("Parser", "payload" , payload)
        log.info("Parser", "decoded into" , data, "ERROR: ", err)
        cmd = data.command_name
        log.info("Parser", "decoded.command", data.command_name)
        cmd_handler(data)
    else 
        log.error("Parser", "payload", paylod, "failed to decoed")
    end


    sys.publish("REPORT")
    -- if command => poweroff , syspublish poweroff
    -- if command => input, syspublish input payload


end

function MQTT.run()
    log.setLevel(2)
    if wlan.ready() then
    local client_id,user_name,password = iotauth.iotda(device_id,device_secret)
    log.info("iotda",client_id,user_name,password)
    
    -- mqtts device join
    mqttc = mqtt.create(nil,"9ceb993f1d.st1.iotda-device.cn-south-1.myhuaweicloud.com", MQTT.port)

    mqttc:auth(client_id,user_name,password)
    mqttc:keepalive(30) -- 默认值240s
    mqttc:autoreconn(true, 3000) -- 自动重连机制

    mqttc:on(function(mqtt_client, event, data, payload)
        -- 用户自定义代码
        log.info("mqtt", "event", event, mqtt_client, data, payload)
        if event == "conack" then
            sys.publish("conack")
            mqtt_client:subscribe("conack")
        elseif event == "recv" then
            log.info("mqtt", "command", "topic", data, "payload", payload)
            handle(payload)
        elseif event == "disconnect" then
            log.info("mqtt", "disconnect",  data, payload)
        elseif event == "sent" then
            log.info("mqtt", "sent finished", data, payload)
        end
    end)

    mqttc:connect()
	sys.waitUntil("disconnect")
    while true do
        -- mqttc自动处理重连
        local ret, topic, data, qos = sys.waitUntil("mqtt_pub", 30000)
        if ret then
            if topic == "close" then break end
            mqttc:publish(topic, data, qos)
        end
    end
    mqttc:close()
    pm.request(LIGHT)
    mqttc = nil

  else 
    log.warn("MQTT", "IP not ready")
  end
end


function MQTT.report_publish()
	local topic = "/ddsdemo/report"
    local report = {}

    report.mcu = rtos.bsp()
    report.dds_maxfreq = 20000000.0
    report.dds_maxvolt = 9000.0
    report.battery = "90%"

	local payload = json.encode(report)
    local decoded = json.decode(payload)
    log.info("json", "decoded reponse", decoded)
    log.info("json", "decoded battery", decoded.battery)
	local qos = 1
    log.info("mqtt", "publish", payload) 

    local result, data = sys.waitUntil("REPORT")

    while true do
        sys.wait(10000)
        if mqttc and mqttc:ready() then
            local pkgid = mqttc:publish(topic, payload, qos)
        end
    end
end

return MQTT
