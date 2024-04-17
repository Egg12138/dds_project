
PROJECT = "lua_test"
VERSION = "0.0.1"


_G.sys = require("sys")
require("sysplus")



_G.NUMS = 10
function where_is_the_seeker_after_zbuff_creat()
  local buf_empty = zbuff.create(NUMS, 0x11)
  buf_empty:seek(0)
  local data = buf_empty:readU8()
  local cnt = 0
  while cnt < NUMS do
    log.info("ZBUFF-TEST", data, tostring(data):toHex())
    cnt = 1 + cnt
    data = buf_empty:readU8()
  end
end

sys.taskInit(function()
  log.info("TEST", "Start")
  where_is_the_seeker_after_zbuff_creat()
end
)

sys.run()
