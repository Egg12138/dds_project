local rtchelper = {}

function rtchelper.display_time()
  log.info("os.date()", os.date())
  local rtc_time = rtc.get()
  log.info("rtc", json.encode(rtc_time))
end

function rtchelper.set_time(year,month,day)
  sys.wait(2000)
  
  rtc.set({year = year, month = month, day = day, min = 0, sec = 0})
end


return rtchelper