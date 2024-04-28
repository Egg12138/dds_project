-- TODO json and other data type parse

require("sys")

local notice = require("log_context")
local logs = notice.zhlog

MCU = require("mcu_defs")
DATA_WIDTH = 8 -- 8 bits
SPIO_BAUD = MCU.C3.spiClk


local HANDLER = {}

-- 命令缓冲区, SPI Commands buffer
-- all elements are numbers
-- 或者用zbuffer, string? 
-- width: 32bits/64bits
HANDLER.cmds_buffer = {}

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
    log.error("SPI-SETUP", logs.spi_creat_err)
    log.error("SPI-SETUP", logs.re)
    sys.wait(200)
    HANDLER.setup_spi()
  else
    log.info("SPI-SETUP", logs.spi_creat_ok)
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
  local fr1_init_buf = 
  stirng.char(0xd3, 0x00, 0x00)
  -- 0b11010011 => FR1[23:16]: VCO gain enable,  PLL divider ratio = 0b10100 = 20, Charge Pump = 0b11 = Max
  SPIOBJ:send(fr1_init_buf)
  sys.wait(500)

  local fr2_init_buf = 
  string.char(0x00, 0x00)
  SPIOBJ:send(fr2_init_buf)
  sys.wait(500)

  local cfr_init_buf = 
  string.char(0x00, 0x03,0x14)
  SPIOBJ:send(cfr_init_buf)
  CS(gpio.HIGH)

  IO_update()

  log.info("DDS initializer", logs.fin)

end

---  reset the DDS IO buffer
function HANDLER.reset_DDSBuffers()
  log.info("DDS Buffer reset", logs.begin)
  gpio.set(MCU.C3.SYNC, gpio.HIGH)
  gpio.set(MCU.C3.SYNC, gpio.LOW)
  log.info("DDS Buffer reset", logs.fin)
end

function HANDLER.reset_DDS()
  log.info("DDS reset", logs.begin)
  gpio.set(MCU.C3.RST, gpio.HIGH)
  sys.wait(10)
  gpio.set(MCU.C3.RST, gpio.LOW)
  log.info("DDS reset", logs.fin)
end

function HANDLER.IO_update()
  log.info("IOUpdate", logs.begin)
  gpio.set(MCU.C3.UPD, gpio.HIGH)
  gpio.set(MCU.C3.UPD, gpio.LOW)
  log.info("IOUpdate", logs.begin)
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
    local sent = SPIOBJ:transfer(spi_data)
    log.info("SPI-SENT", sent)
  end
  IO_update()
  CS(gpio.LOW)

end

local function buffer2spi(cmd)
  local spi_data = 0
  local flag = true
  CS(gpio.LOW)
  for i = 56, -9, -8 do
    spi_data = cmd >> i -- 使用bit库进行位与操作，假设bit库已加载
    if flag and spi_data > 0x00 then
        if spi_data < 0x19 then
            local sent = SPIOBJ:transfer(spi_data) 
            flag = false
        elseif spi_data == 0x20 then
            spi_data = 0x00
            sent = SPIOBJ:transfer(spi_data) -- 同样，sent未被使用
            flag = false
        end
    else
        local sent = SPIOBJ:transfer(spi_data) -- 这里sent同样未被使用
        log.info("SPI Buffer to SPI", logs.fin)
        log.info("SPI Transfer", "sent", sent)
    end
  end

end

function HANDLER.buffer2spi_by_idx(cmd_idx)
  local spi_data = 0
  CS(gpio.LOW)
  --[==[
   TODO:   future
  --]==]
  buffer2spi(HANDLER.cmds_buffer[cmd_idx])

  CS(gpio.HIGH)
end

  --[==[
   NOTICE:   检查width
  --]==]
function HANDLER.store_spicmds(parsed_cmds)
  for parsed_cmd in parsed_cmds  do
    table.insert(HANDLER.cmds_buffer, parsed_cmd)
  end

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
