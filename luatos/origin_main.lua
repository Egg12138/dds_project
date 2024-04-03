PROJECT = "DDS-DEMO"
VERSION = "0.1.2"
_G.sys = require("sys")
require("sysplus")
local checker = require("checks")

if wdt then
	wdt.init(9000)
	sys.timerLoopStart(wdt.feed, 3000)
end

-- TODO: finished this...
sys.taskInit(function ()
	--    INIT

	checker.machin_check()

	checker.http_checks()
	-- meminfo

	-- checker.screen_checks()


			-- try connect to 
				-- wifi init
				-- bluetooth init 
				-- screen init

			-- MAIN loop :
				-- sys.waitUntil Message
					-- callback: 
						-- send to DDS module
						-- screen display
						-- waiting for DDS feedback
						-- screen display

end )

sys.subscribe("WLAN_SCAN_DONE", function()
	local result = wlan.scanResult()
	_G.scan_result = {}
	for k,v in pairs(result) do
		log.info("scan",
			(v["ssid"] and #v["ssid"] > 0)
			and
			v["ssid"] or "[隐藏SSID]", v["rssi"], (v["bssid"]:toHex()))
		if v["ssid"] and #v["ssid"] > 0 then
				table.insert(_G.scan_result, v["ssid"])
		end
	end
	log.info("scan", "aplist", json.encode(_G.scan_result))
end
)

sys.subscribe("IP_READY", function()
	log.info("wlan", "conected", ">>>>>>>>>>>>>>")
	sys.taskInit(function()
		sys.wait(1000)
		-- 以下是rtkv库的模拟实现, 这里就不强制引入rtkv了
		local token = mcu.unique_id():toHex()
		local device = wlan.getMac()
		local params = "device=" .. device .. "&token=" .. token
		params = params .. "&key=ip&value=" .. (socket.localIP())
		local code = http.request("GET", "http://rtkv.air32.cn/api/rtkv/set?" .. params, {timeout=3000}).wait()
		log.info("上报结果", code)
end)

end
)



sys.run()