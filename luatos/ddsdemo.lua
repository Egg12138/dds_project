PROJECT = "demo-9910"
VERSION = "0.1.1"
Provider = "PROV_eggs"
Password = "liyuan11328"

_G.sys = require("sys")
srv = require "luatos.tcpsrv"
spi = require "luatos.dds_spi"

wifihelper = require "luatos.wifi-utils"

if mcu then
  mcu.setClk(40) 
  -- mcu.setClk(240)
end

-- local DDS_WIFI = {}


CSB_DDS_Pin = 15
UPD_DDS_Pin = 5
RST_DDS_Pin = 4
SYSFREQ_DDS = 800137209

HIGH = gpio.HIGH
LOW = gpio.LOW
UP = gpio.PULLUP
DOWN = gpio.PULLDOWN

function init_DDS()
  -- IMPL 
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



