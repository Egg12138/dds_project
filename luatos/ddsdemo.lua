PROJECT = "demo-9959"
VERSION = "0.1.1"
Provider = "PROV_eggs"
Password = "liyuan11328"

_G.sys = require("sys")
srv = require "luatos.tcpsrv"
spi = require "luatos.dds_spi"
local ffi = require "ffi"

---@diagnostic disable-next-line: different-requires
wifihelper = require "luatos.wifi-utils"

if mcu then
  mcu.setClk(40) 
  -- mcu.setClk(240)
end

-- local DDS_WIFI = {}

-- registers 
-- ID = register addr
CSR_ID    = 0x00   --CSR  Channel Select Register(通道选择寄存器)                1 Bytes
FR1_ID    = 0x01   --FR1  Function Register 1(功能寄存器1)                       3 Bytes
FR2_ID    = 0x02   --FR2  Function Register 2(功能寄存器2)                       2 Bytes
CFR_ID    = 0x03   --CFR  Channel Function Register(通道功能寄存器)              3 Bytes
CFTW0_ID  = 0x04   --CTW0 Channel Frequency Tuning Word 0(通道频率转换字寄存器)  4 Bytes
CPOW0_ID  = 0x05   --CPW0 Channel Phase Offset Word 0(通道相位转换字寄存器)      2 Bytes
ACR_ID    = 0x06   --ACR  Amplitude Control Register(幅度控制寄存器)             3 Bytes
LSRR_ID   = 0x07   --LSR  Linear Sweep Ramp Rate(通道线性扫描寄存器)             2 Bytes
RDW_ID    = 0x08   --RDW  LSR Rising Delta Word(通道线性向上扫描寄存器)          4 Bytes
FDW_ID    = 0x09   --FDW  LSR Falling Delta Word(通道线性向下扫描寄存器)         4 Bytes

-- Pins

CSB_DDS_Pin = 15
UPD_DDS_Pin = 5
RST_DDS_Pin = 4
SYSFREQ_DDS = 800137209

HIGH = gpio.HIGH
LOW = gpio.LOW
UP = gpio.PULLUP
DOWN = gpio.PULLDOWN

-- uint8 *
CSR_DATA0 = {0x10} 
CSR_DATA1 = {0x20}
CSR_DATA2 = {0x40}
CSR_DATA3 = {0x80}

FR2_DATA = {[0] = 0x00, [1] = 0x00}
CFR_DATA = {0x00, 0x03, 0x02}
CPOW0_DATA = {0x00, 0x00}
LSRR_DATA = {0x00, 0x00}
RDW_DATA = {0x00, 0x00, 0x00, 0x00}
FDW_DATA = {0x00, 0x00, 0x00, 0x00}

-- uint32 *
SinFreq = {10000000, 10000000, 200000000, 40000}
SinAmp = {9215, 9215, 9215, 9215}
SinPhase = {0, 4095, 4095*3, 4095*2}

-- IO port operation macro definition
function BITBAND(addr, bitnum) 
  return ((addr & 0xF0000000)+0x2000000+((addr &0xFFFFF)<<5)+(bitnum<<2)) 
end

function MEM_ADDR(addr)  
  return  addr 
end
function BIT_ADDR(addr, bitnum)   
  return MEM_ADDR(BITBAND(addr, bitnum)) 
end

GPIOD_ODR_ADDR = 0x40020C14
GPIOD_IDR_ADDR = 0x40020C10

-- TODO lua ffi is needed

-- @description => DDS module: AD9959
function init_DDS()
  -- IMPL 
  FR1_DATA = {0xd3, 0x00, 0x00}
  GPIO_INIT = {}
  GPIO_INIT.pin = 0x4fff

end

function setup_DDS()
  gpio.set(RST_DDS_Pin, HIGH)
  sys.wait(100)   
  gpio.set(RST_DDS, LOW)



end

function nwriteDDSreg(addr, bytes, datas)
  gpio.set(CSB_DDS, LOW)
end


-- TODO: refactor the server function
function server()
  return tcpsrv.setup()
end




function dds_main()
  if wlan and wlan.connect then
    wifihelper.setup(STATION)
  elseif mobile then
    log.info("mobile", "auto connect")
  elseif socket then
    sys.timeStart(sys.publish, 1000, "Socket_Simulation")
  else
    while 1 do
      sys.wait(1000)
      log.warn("Wifi BSP","the BSP may not be adated to the network layor")
    end
  end



  init_DDS()




end


sys.taskInit(dds_main)
sys.run()



