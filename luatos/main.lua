
PROJECT = "DDS-Demo"
VERSION = "0.1.7"

_G.sys = require("sys")

checker = require("checks")
wifi = require("wifi-manager")
mqtthelper = require("mqtts")

DATA_STREAM = ""
DDS = require("dds_defs")
MCU = require("mcu_defs")
COMMUNICATION_MODE = "MQTT" -- MQTT, Socket, ...
-- wdt.init(10000) -- watch dog timer: 10s

DATA_WIDTH = 8 -- 8 bits
SPIO_BAUD = MCU.C3.spiClk

function get_spi_modes(mode)
  mode = mode or 0 -- default mode is SPI MODE0
  -- grammar works in Lua 5.3
  cpha = (mode & 0x3) >> 1
  cpol = (mode & 0x2)
  return cpha, cpol
end

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

  wifi.simpleRun()
  MCU.init("esp32c3")


  cpha, cpol = get_spi_modes(3)
  -- hardware SPI
  -- spi.setup(spi.SPI_3, MCU.C3.SS, cpha, cpol, DATA_WIDTH, SPIO_BAUD, spi.MSB, 1, DUAL)
  -- software SPI
  if not spi.createSoft(MCU.C3.SS, MCU.C3.MOSI,MCU.C3.MISO, MCU.C3.SCLK,cpha,cpol, 
  DATA_WIDTH, SPIO_BAUD, spi.MSB, spi.master, spi.full) then
    log.error("SPI-SETUP","create Soft failed")
  else 
    log.info("SPI-SETUP","create Soft success")
  end

end

function main()
  init_system()
  while true do
    log.info("main", "loop")
    sys.wait(5000)

    if COMMUNICATION_MODE == "Socket" then
      socket_listen()
      client = server.available()
      if client then
        while client.connected() do
          local data = client.read()
          log.info("Socket read", "data", data)

          data_handler(data)


        end 
      end
      elseif COMMUNICATION_MODE == "MQTT"  then
        sys.taskInit(mqtthelper.run)
    end

  end
end

-- GC per 5 seconds
sys.timerLoopStart(function()
  collectgarbage("collect")
end
, 5000)

sys.taskInit(main)

sys.subscribe("START", function()
  log.info("wlan power save", "canceld")
  wlan.powerSave(wlan.PS_NONE)
end)

sys.subscribe("PAUSE", function()
  log.info("wlan power save", "pause. waiting for new commands")
  wlan.powerSave(wlan.PS_MAX_MODEN)
end
)





sys.run()