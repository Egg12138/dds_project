local AD9959 = {}

AD9959.commands = {
  "input",
  "sync",
  "init",
  "reset",
  "poweroff",
  "update",
  "report",
  "spi",
  "listreset",
  "listmode",
  "listlength",
}
AD9959.pins = {}
AD9959.regs = {}
AD9959.pins.pin_IO_intr = 19 -- changable
AD9959.max_cycles = 120000
AD9959.cmdscapcaity = 1000

-- Define pins for SPI communication and AD9959 control
-- ESP32C3-core Luatos
-- AD9959 pins definitions
AD9959.pins.SYNC_IN = 1
AD9959.pins.SYNC_OUT = 2 -- Used to Synchronize Multiple AD9959 Devices. Connects to the SYNC_IN pin of the slave AD9959 devices. 7
AD9959.pins.MASTER_RESET = 3 -- Active High Reset Pin. Asserting the MASTER_RESET pin forces the AD9959 internal registers to their default state, as described in the Register Maps and Bit Descriptions section. 4
AD9959.pins.PWR_DWN_CTL = 4 -- External Power-Down Control. 5
AD9959.pins.AVDDs = {5,7,11,15,19,21,26,31,33,37,39} -- Analog Power Supply Pins (1.8 V). 6
AD9959.pins.AGNDs = {6,10,12,16,18,20,25,28,32,34,38} -- Analog Ground Pins. 7
AD9959.pins.DVDDs = {45, 55} -- Digital Power Supply Pins (1.8 V). 46
AD9959.pins.DGNDs = {44,46} -- Digital Power Ground Pins. 57
AD9959.pins.CH2_IOUT = 8 -- True DAC Output. Terminates into AVDD. 9
AD9959.pins.CH2_IOUT_c = 9 -- Complementary DAC Output. Terminates into AVDD.
AD9959.pins.CH3_IOUT = 13 -- True DAC Output. Terminates into AVDD. 14
AD9959.pins.CH3_IOUT_c = 14 -- Complementary DAC Output. Terminates into AVDD.
AD9959.pins.DAC_RSET = 17 -- Establishes the Reference Current for All DACs. A 1.91 kΩ resistor (nominal) is connected from Pin 17 to AGND. 18
AD9959.pins.REF_CLK_c = 22 -- Complementary Reference Clock/Oscillator Input. When operated in single-ended mode, this pin should be decoupled to AVDD or AGND with a 0.1 μF capacitor. 23
AD9959.pins.REF_CLK = 23
AD9959.pins.CLK_MODE_SEL = 24 -- Control Pin for the Oscillator Section. Caution: Do not drive this pin beyond 1.8 V. When high (1.8 V), the oscillator section is enabled to accept a crystal as the REF_CLK source. When low, the oscillator section is bypassed.
AD9959.pins.LOOP_FILTER = 27 -- Connects to the external zero compensation network of the PLL loop filter. Typically, the network consists of a 0 Ω resistor in series with a 680 pF capacitor tied to AVDD.
AD9959.pins.CH0_IOUT_c = 29
AD9959.pins.CH0_IOUT = 30
AD9959.pins.CH1_IOUT_c = 35
AD9959.pins.CH1_IOUT = 36
AD9959.pins.P0 = 40
AD9959.pins.P1 = 41
AD9959.pins.P2 = 42
AD9959.pins.P3 = 43
AD9959.pins.IO_UPDATE = 46
AD9959.pins.CS_c = 47
AD9959.pins.SCLK = 48
AD9959.pins.DVDD_IO = 49
AD9959.pins.SDIO_0 = 50
AD9959.pins.SDIO_1 = 51
AD9959.pins.SDIO_2 = 52
AD9959.pins.SDIO_3 = 53
AD9959.pins.SYNC_CLK = 54

-- AD9959 registers
AD9959.regs.CSR = 0x00
AD9959.regs.FR1 = 0x01
AD9959.regs.FR2 = 0x02
AD9959.regs.CFR = 0x03
AD9959.regs.CFTW = 0x04
AD9959.regs.ACR = 0x06
AD9959.regs.MultiplierEnable = 0x1000






return AD9959