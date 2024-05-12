
PROJECT = "DDS-Demo"
VERSION = "0.1.7"

-- wdt.init(10000) -- watch dog timer: 10s


  --[=====[
  ```c++17
  public:
  SPIClass(uint8_t spi_bus = HSPI);
  ~SPIClass();
  void begin(int8_t sck = -1, int8_t miso = -1, int8_t mosi = -1, int8_t ss = -1);
  void end();
  ```
  the Luatos provides a similar API
  ```c++17
  class SPISettings {
    public:
      SPISettings()
        : _clock(1000000), _bitOrder(SPI_MSBFIRST), _dataMode(SPI_MODE0) {}
      SPISettings(uint32_t clock, uint8_t bitOrder, uint8_t dataMode)
        : _clock(clock), _bitOrder(bitOrder), _dataMode(dataMode) {}
      uint32_t _clock;
      uint8_t _bitOrder;
      uint8_t _dataMode;
    };
  ```
  --]=====]
  -- TODO 确定 SPIO mode
  
function init_system() 

  wifi.simple_run()
  MCU.init("esp32c3")

  handler.setup_spi()
  handler.init_DDS()
  
end

function datapkg_parser(client, data) 

  datapkg = json.decode(data)

  if datapkg then
    assert(datapkg.command_name, "json decoded incorrectly, the field `command_name` shoule be found!")
    cmd = datapkg.command_name
    if not handler.contains(DDS.commands, cmd) then
      log.error("DDS-HANDLER", "command not found", cmd)
      return
    end
    if cmd == "input" then 
      handler.set_input(datapkg.paras)
    elseif cmd == "spi" then

      log.info("CmdHandler", "spi cmds", paras)  
      spi_cmd = paras
      handler.spi_cmds_transfer(spi_cmd)

    elseif cmd == "init" then
      handler.init_DDS()
      log.info("DataHandler", "DDS init")
    elseif cmd == "reset" then
      -- software reset 
      reset_DDS()
      log.info("DataHandler", "DDS reset")
    elseif cmd == "update" then
      IO_update()
      log.info("DataHandler", "DDS updated")
    elseif cmd == "sync" then
    elseif cmd == "poweroff" then
    elseif cmd == "report" then
      -- dds_info = "AD9959 & ESP32C3"
      client.send(dds_info)
      log.info("DataHandler", "Report is not supported well currently"    )
    -- elseif cmd == "listlength" then
      -- num_cmds = tonumber(paras)
    elseif cmd == "listmode" or
      cmd == "listreset" or
      cmd == "listlength"
    then
      log.warn("DataHandler", "List Mode is not supported currently")
    end
      
  else
    log.warn("Datahandler", "json decoded failed", data)
  end
end

-- GC per 5 seconds

sys.taskInit(function()

  init_system()
    log.info("main", "loop")
    sys.wait(5000)

    if COMMUNICATION_MODE == "Socket" then
      socket_listen()
      client = server.available()
      if client.connected() then
          local data = client.read_whole_packet()
          log.info("Socket read", "data", data)

          datapkg_parser(data)

      end
    elseif COMMUNICATION_MODE == "MQTT"  then
        sys.taskInit(iot.run)
    end




end)

sys.subscribe("START", function()
  log.info("wlan power save", "canceld")
  wlan.powerSave(wlan.PS_NONE)
end)

sys.subscribe("PAUSE", function()
  log.info("wlan power save", "pause. waiting for new commands")
  wlan.powerSave(wlan.PS_MAX_MODEN)
end
)



function d() 
   client_id, user_name, password = iotauth.iotda(
      iot.device_id,
      iot.device_secret
    )

    log.info("IoTDA", client_id, user_name, password)

    mqttc = mqtt.create(nil, iot.iot_url, iot.port)

    mqttc:auth(client_id, user_name, password)
    mqttc:keepalive(30)
    mqttc:autoreconn(true, 3000)
    mqttc:on(function(mqtt_client, event, data, payload, metas)
    log.info("mqtt", logs.iot_authorize_ok)
    log.info("mqtt", "event", event, mqtt_client, data, payload)
    if event == "conack" then
      sys.publish("MQTT-conack")
      mqtt_client:subscribe(iot.topics.setinput_dSpP, 1)
    elseif event == "sent" then
      log.info("mqtt:sent", logs.fin, "mqtt message id", data, payload)
    elseif event == "recv" then
      log.info("mqtt:receive", "command", "topic", data, "payload", payload)
      --[==[
      -- notice: metas: 
      --           -- qos: 0, 1, 2
      --           -- retain: 0, 1
      --           -- duo: 0, 1
      --]==]
      log.info("mqtt:received metas", metas)
    elseif event == "disconnect" then
      mqttc:disconnect()
    else 
      log.error("mqtt", event, logs.unsupported )
      log.info("mqtt", "received", data, payload, metas)
    end
end

sys.run()