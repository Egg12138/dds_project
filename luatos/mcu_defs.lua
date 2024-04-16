_G.sys = require("sys")
local MCU = {}
MCU.C3 = {}
-- TODO: rewrite pin numbers
-- 全部为 GPIO 编号。 esp32的开发板中已经将PWB,V,GND 之外的所有口统一gpio命名了。
MCU.C3.INTR = 6 -- Listen to the IO Update (UART need?)
MCU.C3.SCLK = 2      -- SPI Clock 与 GPIO02 复用, I/O + High
MCU.C3.MOSI = 3      -- SPI data out, I/O + High
MCU.C3.MISO = 10      -- SPI data IN, 与 GPIO03 服用,  I/O+High
MCU.C3.SS   = 7      -- slave select (similar to CS in SPI)
MCU.C3.RST_DDS  = 11         -- reset pin of the AD9959 (不是 reset MCU!)
MCU.C3.UPD  = 4      -- update pin of the AD9959
MCU.C3.SYNC = 5      -- communication reset pin of the AD99590
MCU.C3.spiClk = 20000000  -- SPI clock frequency: 20 MHz
-- C3.hspi = nil     -- uninitalised reference to an SPI object

-- 不可用
MCU.S3 = { 
  INTR = 6, SCLK = 18, MOSI = 17, MISO = 16,
  SS = 14, RST_DDS = 15, UPD = 14, SYNC = 9,
  spiCLK = 20000000
}

INPUT = 0x0
OUTPUT = 0x1
  -- IO12, IO13 => LED D4/D5, 
  -- IO9 => BOOT, 低电平有效
  -- IO18, 19 => for USB 

function esp32c3_init() 
  -- setup to **OUTPUT** mode, `setup` returns closure to set(output)/get(input) the level
  cs = gpio.setup(MCU.C3.SS, OUTPUT)
  upd = gpio.setup(MCU.C3.UPD, OUTPUT)
  rst = gpio.setup(MCU.C3.RST_DDS, OUTPUT)
  sync = gpio.setup(MCU.C3.SYNC, OUTPUT)
  intr = gpio.setup(MCU.C3.INTR, INPUT, PULLUP) -- 内部 PULLUP

end

function esp32s3_init()
  cs = gpio.setup(MCU.S3.SS, OUTPUT)
  upd = gpio.setup(MCU.S3.UPD, OUTPUT)
  rst = gpio.setup(MCU.S3.RST_DDS, OUTPUT)
  sync = gpio.setup(MCU.S3.SYNC, OUTPUT)
  intr = gpio.setup(MCU.S3.INTR, INPUT, PULLUP) -- 内部
end

function MCU.init(board)
  if board == "esp32c3" or board == "ESP32C3" then
    esp32c3_init()
  elseif board == "esp32s3" or board == "ESP32S3" then
    esp32s3_init()
  else
    log.error("MCU-INIT", "Unsupported board, only ESP32C3(luatos) and ESP32S3(luatos) are supported",board)
  end
end


return MCU