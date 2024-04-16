PROJECT = "demo-9959"
VERSION = "0.1.1"
Provider = "PROV_eggs"
Password = "liyuan11328"

_G.sys = require("sys")
srv = require("luatos.connection.tcpsrv")
spi = require("luatos.rubish.dds_spi")

local ffi = require("ffi")

log.setLevel(1)

---@diagnostic disable-next-line: different-requires
wifi = require("luatos.connection.wifimanager")

if mcu then
	mcu.setClk(40)
	-- mcu.setClk(240)
end

-- local DDS_WIFI = {}

-- registers
-- ID = register addr
CSR_ADDR = 0x00 --CSR  Channel Select Register(通道选择寄存器)                1 Bytes
FR1_ADDR = 0x01 --FR1  Function Register 1(功能寄存器1)                       3 Bytes
FR2_ADDR = 0x02 --FR2  Function Register 2(功能寄存器2)                       2 Bytes
CFR_ADDR = 0x03 --CFR  Channel Function Register(通道功能寄存器)              3 Bytes
CFTW0_ADDR = 0x04 --CTW0 Channel Frequency Tuning Word 0(通道频率转换字寄存器)  4 Bytes
CPOW0_ADDR = 0x05 --CPW0 Channel Phase Offset Word 0(通道相位转换字寄存器)      2 Bytes
ACR_ADDR = 0x06 --ACR  Amplitude Control Register(幅度控制寄存器)             3 Bytes
LSRR_ADDR = 0x07 --LSR  Linear Sweep Ramp Rate(通道线性扫描寄存器)             2 Bytes
RDW_ADDR = 0x08 --RDW  LSR Rising Delta Word(通道线性向上扫描寄存器)          4 Bytes
FDW_ADDR = 0x09 --FDW  LSR Falling Delta Word(通道线性向下扫描寄存器)         4 Bytes

-- IO port operation macro definition
function BITBAND(addr, bitnum)
	return ((addr & 0xF0000000) + 0x2000000 + ((addr & 0xFFFFF) << 5) + (bitnum << 2))
end

function MEM_ADDR(addr)
	return addr
end

function BIT_ADDR(addr, bitnum)
	return MEM_ADDR(BITBAND(addr, bitnum))
end

GPIOD_ODR_ADDR = 0x40020C14
GPIOD_IDRR_ADDR = 0x40020C10

function PDOut(n, v)
	BIT_ADDR(GPIOD_ODR_ADDR, n)
end

function PDin(n, v)
	BIT_ADDR(GPIOD_IDRR_ADDR, n)
end

function SCLK(v)
	PDOut(0,v)
end
function CS(v)
	PDout(1,v)
end
function UPDATE(v)
	PDOut(2,v)
end
function SDIO0(v)
	PDout(3,v)
end
function PS0(v)
	PDout(4,v)
end
function PS1(v)
	PDout(5,v)
end
function PS2(v)
	PDout(6,v)
end
function PS3(v)
	PDout(7,v)
end
function SDIO1(v)
	PDout(8,v)
end
function SDIO2(v)
	PDout(9,v)
end
function SDIO3(v)
	PDout(10,v)
end
function AD9959_PWR(v)
	PDout(11,v)
end
function RESET(v)
	PDout(14,v)
end

CSR = gpio.setup(CSR_ADDR, HIGH)

-- Pins

CSB_DDS_Pin = 15
UPD_DDS_Pin = 5
RST_DDS_Pin = 4
SYSFREQ_DDS = 800137209

HIGH = gpio.HIGH
LOW = gpio.LOW
PULLUP = gpio.PULLUP
PULLDOWN = gpio.PULLDOWN
OUT0 = 0 -- gpio out mode, with init volt = 0 (LOW)
OUT1 = 1 -- init volt = 1 (HIGH)
IN = nil -- gpio input mode

buff = zbuff.create(512, 0)

buff:seek(0)
-- uint8 * NOTICE: data:byte(FROM 0!!!!!!!!!)
CSR_DATA0 = buff:write(0x10)
CSR_DATA1 = buff:write(0x20)
CSR_DATA2 = buff:write(0x40)
CSR_DATA3 = buff:write(0x80)

FR2_DATA = buff:write(0x00, 0x00)
CFR_DATA = buff:write(0x00, 0x03, 0x02)
CPOW0_DATA = buff:write(0x00, 0x00)
LSRR_DATA = buff:write(0x00, 0x00)
RDW_DATA = buff:write(0x00, 0x00, 0x00, 0x00)
FDW_DATA = buff:write(0x00, 0x00, 0x00, 0x00)

log.info("zbuff", "expected cur", 21, buff:seek(0, zbuff.SEEK_CUR))

-- uint32 *
SinFreq = string.pack("<L", 10000000, 10000000, 200000000, 40000)
SinAmp = string.char(9215, 9215, 9215, 9215)
SinPhase = string.char(0, 4095, 4095 * 3, 4095 * 2)

-- TODO lua FFI is needed

function writedata_AD9959(register_addru8, regs_numu8, datau8, tempu8)
	control_valueu8 = 0
	value2writeu8 = 0
	register_idu8 = 0
	i = 0
    control_valueu8 = register_idu8;
    SCLK(0)
    CS(0)
    
end

-- @description => DDS module: AD9959
function init_DDS()
	-- IMPL
	-- TODO rename GG => GPIO_INIT
	FR1_DATA = { 0xd3, 0x00, 0x00 } -- uint8
	GG = {}
	GG.pin = 0x4fff -- TODO !
	local gpiod = gpio.setup(
		GG.pin,
		OUT,
		PULLUP
		-- trigger-default: gpio.BOTH,
	)

	init_ddsio()
	init_reset()
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

function display_screen()
	print("LVGL screen")
end
-- @description 先只做 wifi
function dds_main_pesudo_code()
	display_screen()

	if wlan and wlan.connect then
		wifi.run()
	elseif mobile then
		log.info("mobile", "auto connect")
	else
		while 1 do
			sys.wait(1000)
			log.warn("Wifi BSP", "the BSP may not be adated to the network layor")
		end
	end

	-- wifi only

	init_DDS()
end

sys.taskInit(dds_main)
sys.run()
