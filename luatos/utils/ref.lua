-- NOTICE: 
-- 全部代码为参考注意的片段!

-- gpio irq: 
local gpio_pin = 7
gpio.debounce(gpio_pin, 100)
gpio.setup(gpio_pin, function()
  log.info("gpio", "PA10")
end, gpio.PULLUP)
