-- TODO json and other data type parse

require("sys")
AD9959 = require("ddss")

local notice = require("log_context")
local logs = notice.zhlog
local parser = require("parser")
MCU = require("mcus")
DATA_WIDTH = 8 -- 8 bits
SPIO_BAUD = MCU.C3.spiClk

CMDNAMES = {
  "poweroff",
  "scan",
  "input",
  "report",
  "init",
  "spi",
  "sync",
  "reset",
}

local HANDLER = {}

-- 命令缓冲区, SPI Commands buffer
-- all elements are numbers
-- 或者用zbuffer, string? 
-- width: 32bits/64bits
HANDLER.cmds_buffer = {}

local function get_spi_modes(mode)
  mode = mode or 0 -- default mode is SPI MODE0
  -- grammar works in Lua 5.3
  cpha = (mode & 0x3) >> 1
  cpol = (mode & 0x2)
  return cpha, cpol
end


---  reset the DDS IO buffer
local function reset_DDSBuffers()
  log.info("DDS Buffer reset", logs.begin)
  gpio.set(MCU.C3.SYNC, gpio.HIGH)
  gpio.set(MCU.C3.SYNC, gpio.LOW)
  log.info("DDS Buffer reset", logs.fin)
end

local function reset_DDS()
  log.info("DDS reset", logs.begin)
  gpio.set(MCU.C3.RST_DDS, gpio.HIGH)
  sys.wait(10)
  gpio.set(MCU.C3.RST_DDS, gpio.LOW)
  log.info("DDS reset", logs.fin)
end

--- func desc 
--- Each set of communication cycles does not require an I/O Update to be issued
--- IO Update 将 IO 端口缓冲区的数据传送到寄存器。
--- 要让数据生效，IO Update 必须要被发送出去:
--- 即: 
--- 1. issue 一个 master reset 信号
--- 2. 配置 Channel Enable Bits (在Single-tone模式下，所有通道都通用同一个FTW,POW寄存器的位置
--- 也就是每一个通道视角中，FTW, POW寄存器的位置都是 0x04, 0x05)
--- 3. 通过串行IO端口，**编程得到需要的FTW, POW**，给每个激活了的通道。
--- 4. 发送 IO Update信号，在IO Update执行后，所有的通道才会开始输出我们需要的信号
local function IO_update()
  log.info("IOUpdate", logs.begin)
  gpio.set(MCU.C3.UPD, gpio.HIGH)
  gpio.set(MCU.C3.UPD, gpio.LOW)
  log.info("IOUpdate", logs.fin)
  log.info("after local:debug", (CS ~= nil), (UPD ~= nil), (RST ~= nil), (SYNC ~= nil), (INTR ~= nil))
  utils.all_fields(_G)
end


--- func desc
--- @param cmd number 二进制命令内容,一次传一条64bits数据
--- comment  
--- AD9959 的 SERIAL i/o port pin 功能比较多。
--- 引脚序列X = (SCLK, CS^, SDIO0, SDIO1, SDIO2, SDIO3), CS^为低电平使能
--- 其中CS实际为SS 因为SPI并不是单纯的片选。
--- X中各引脚的功能作用与模式相关:(单比特双线串行、但比特三线串行，二比特串行，四比特串行)
--- 串行IO口的最大同步时钟频率是200MHz，于是我们充分使用四个SDIO端口，就可以达到
--- SDIO0,1,2,3这四个来达到极限数据吞吐量到800Mbps
--- 一个串行通信周期有两个阶段，
--- * 第一个阶段是指令周期：将指令字节写道AD9959中。指令字节的每一位都在对应的**SCLK上升沿**被寄存。
--- 这个周期的指令字包括地址寄存器的串行地址。
--- * 第二个阶段是IO阶段。在这个阶段发生串口控制器与串行端口缓冲区之间的数据传输。
--- **SCLK上升沿数量和寄存器宽有关**, FR1宽24bits，则第二阶段的就需要传输3字节。每一个指令字节
--- 传输一个字节，在三个字节都传完后，整个通信周期才完成。
local function buffer2spi(cmd)
  local spi_data = 0
  local flag = true
  CS(gpio.LOW)
  -- 64bits=8Byte宽的指令，buffer存放8条。
  for i = 56, -9, -8 do
    spi_data = cmd >> i
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
        log.info("SPI Transfer", "sent", sent, logs.fin)
    end
  end

end

--- func desc
--- @param cmd_idx number : 二进制命令的缓冲区索引
local function buffer2spi_by_idx(cmd_idx)
  local spi_data = 0
  CS(gpio.LOW)
  buffer2spi(HANDLER.cmds_buffer[cmd_idx])
  CS(gpio.HIGH)
end

local function store_spicmds(cmd_codes)
  for binary_cmd in cmd_codes  do
    table.insert(HANDLER.cmds_buffer, binary_cmd)
  end
end

local function DDS_into_listmode()
  local list_ele = 0
  gpio.setup(MCU.C3.UPD, INPUT, PULLUP)
  -- ensure gpio has been init!J
  gpio.set(MCU.C3.CS, LOW)
  -- TODO: support memory_spi (spi commands storage)
end

local function set_input(paras)
  log.info("CmdHandler", "set input")
  local freq,volt,phase = paras.freq_hz, paras.volt_mv, paras.phs_oft
  log.info("CmdHandler", "parsed",
    "frequency(Hz)", freq, 
    "voltage(mV)", volt,
    "phase(degree)", phase)
end

local function raw_handle_command(cmdj, paras)
  if cmd == "set_input" then
    set_input(paras)
  elseif cmd == "set_output" then
    set_output(paras)
  elseif cmd == "set_freq" then
    set_freq(paras)
  elseif cmd == "set_volt" then
    set_volt(paras)
  elseif cmd == "set_phase" then
    set_phase(paras)
  end

end

local function handle_command(cmd, paras)
  if utils.contains(CMDNAMES, cmd) then
    log.info("CmdHandler", cmd)
    raw_handle_command(cmd, paras)
  else
    log.error("CmdHandler", logs.invalid_command, cmd)
    return 
  end
end


function HANDLER.setup_spi()
  cpha, cpol = get_spi_modes(3)
  -- hardware SPI
  -- 手动管理CS, CS => nil
  _G.SPIOBJ = spi.deviceSetup(spi.SPI_2, nil, cpha, cpol, DATA_WIDTH, SPIO_BAUD, spi.MSB, 1, DUAL)
  -- software SPI
  -- _G.SPIOBJ = spi.createSoft(MCU.C3.SS, MCU.C3.MOSI,MCU.C3.MISO, MCU.C3.SCLK,cpha,cpol, 
  -- DATA_WIDTH, SPIO_BAUD, spi.MSB, spi.master, spi.full) then
  _G.CS = gpio.setup(MCU.C3.CS, gpio.HIGH)
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
  gpio.set(MCU.C3.CS, gpio.HIGH)
  gpio.set(MCU.C3.UPD, gpio.LOW)
  gpio.set(MCU.C3.RST_DDS, gpio.LOW)
  gpio.set(MCU.C3.SYNC, gpio.LOW)
  sys.wait(1)
  --[==[
  -- NOTICE: expain why sys.wait(1)
  -- ]==]
  reset_DDS()
  sys.wait(1)
  reset_DDSBuffers()
  sys.wait(1)

  CS(gpio.LOW)
  local fr1_init_buf = 
  string.char(0xd3, 0x00, 0x00)
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

function HANDLER.handle_received(data)
  local cmd_table = parser.json2table(data)
  log.info("HANDLER:decode", data, "decoded into-->", cmd_table)
  utils.all_fields_rec(cmd_table)

  local cmd_type, paras =  parser.type_and_paras(cmd_table)
  log.info("Command Type", cmd_type, "Parameters", paras)
  handle_command(cmd_type, paras)

end

return HANDLER
