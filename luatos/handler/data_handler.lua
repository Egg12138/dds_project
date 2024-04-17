-- TODO json and other data type parse

require("sys")

MCU = require("mcu_defs")
DATA_WIDTH = 8 -- 8 bits
SPIO_BAUD = MCU.C3.spiClk


local HANDLER = {}


local function contains(t,pattern) 
  for k,v in ipairs(t) do
    if v == pattern then
      return true
    end
  end
  return false
end

local function get_spi_modes(mode)
  mode = mode or 0 -- default mode is SPI MODE0
  -- grammar works in Lua 5.3
  cpha = (mode & 0x3) >> 1
  cpol = (mode & 0x2)
  return cpha, cpol
end

function HANDLER.setup_spi()
cpha, cpol = get_spi_modes(3)
  -- hardware SPI
  -- 手动管理CS, CS => nil
  _G.SPIOBJ = spi.deviceSetup(spi.SPI_2, nil, cpha, cpol, DATA_WIDTH, SPIO_BAUD, spi.MSB, 1, DUAL)
  -- software SPI
  -- _G.SPIOBJ = spi.createSoft(MCU.C3.SS, MCU.C3.MOSI,MCU.C3.MISO, MCU.C3.SCLK,cpha,cpol, 
  -- DATA_WIDTH, SPIO_BAUD, spi.MSB, spi.master, spi.full) then
  _G.CS = gpio.setup(MCU.C3.SS, gpio.HIGH)
  if not SPIOBJ then
    log.error("SPI-SETUP","create SPI device failed")
  else 
    log.info("SPI-SETUP","create SPI device success")
  end
end

function HANDLER.init_DDS()
  gpio.set(MCU.C3.SS, gpio.HIGH)
  gpio.set(MCU.C3.UPD, gpio.LOW)
  gpio.set(MCU.C3.RST_DDS, gpio.LOW)
  gpio.set(MCU.C3.SYNC, gpio.LOW)
  sys.wait(1)
  --[==[
  NOTICE: expain why sys.wait(1)
  -- ]==]
  reset_DDS()
  sys.wait(1)
  reset_DDSBuffers()
  sys.wait(1)

  CS(gpio.LOW)
  local fr1_init_buf = stirng.char(0x0D, 0x00, 0x00)
  -- 0b11010011 => FR1[23:16]: VCO gain enable,  PLL divider ratio = 0b10100 = 20, Charge Pump = 0b11 = Max
  SPIOBJ:send(fr1_init_buf)

  local fr2_init_buf = string.char(0x00, 0x00)
  sys.wait(500)
  SPIOBJ:send(fr2_init_buf)

  local cfr_init_buf = string.char(0x00, 0x03,0x14)
  sys.wait(500)
  SPIOBJ:send(cfr_init_buf)
  CS(gpio.HIGH)

  IO_update()

  log.info("DDS initializer", "finished")

end

function HANDLER.reset_DDSBuffers()
  gpio.set(MCU.C3.SYNC, gpio.HIGH)
  gpio.set(MCU.C3.SYNC, gpio.LOW)
  log.info("DDS Buffer", "reset")
end

function HANDLER.reset_DDS()
  gpio.set(MCU.C3.RST, gpio.HIGH)
  sys.wait(100)
  gpio.set(MCU.C3.RST, gpio.LOW)
end

function HANDLER.IO_update()
  log.info("IOUpdate", "setting")
  gpio.set(MCU.C3.UPD, gpio.HIGH)
  gpio.set(MCU.C3.UPD, gpio.LOW)
end

-- [[
  --@parameter: spi_cmd, String from wifi
--]]
function HANDLER.spi_cmds_transfer(spi_cmd)

  local len = string.len(spi_cmd) -- non-null char nums
  local spi_codes = tonumber(spi_cmd)
  local spi_data = zbuff.create()
  if len % 2 == 0 then
    len = 4 * (len - 2) - 8
  else 
    len = 4 * (len - 1) - 8
  end

  CS(gpio.LOW)
  for bit in len, -1, -8 do
    spi_data = (spi_codes >> bit) & 0xff
    if (bit == len) and spi_data > 0x18 then
      spi_data = 0x00
    end
    SPIOBJ:send(spi_data)
  end
  IO_update()
  CS(gpio.LOW)

end

function HANDLER.DDS_into_listmode()
  local list_ele = 0
  gpio.setup(MCU.C3.UPD, INPUT, PULLUP)
  -- ensure gpio has been init!J
  gpio.set(MCU.C3.SS, LOW)
  -- TODO: support memory_spi (spi commands storage)
end

function HANDLER.set_input(paras)
  log.info("CmdHandler", "set input")
  freq,volt,phase = paras.freq_hz, paras.volt_mv, paras.phs_oft
  log.info("CmdHandler", "parsed",
    "frequency(Hz)", freq, 
    "voltage(mV)", volt,
    "phase(degree)", phase)


end
