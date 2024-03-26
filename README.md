# today

## project software demo todos

TODO: 1. arduino 快速测试 wifi 收发 控制ad9910
TODO: 2. luatos API 教程选做
TODO: 3. luatos WIFI API写demo
TODO: 4. backend demo
TODO: 5. html controller

### Frontend

有可能改成 python 实现，最后整个项目可能就会是:



#### layout 

* xmake build
* python/Rust backend
* mobild remote backend (GUI)
* esp32 <--> DDS
* esp32 <--> IoT cloud (WIFI/BLE/CONNECTION)<--> Ends
* edp32 --> Screen (固定占用几个引脚)

build:

TODO: build as a binary

#### command line (git/cargo style)

options/actions: (git/cargo-like style)

* `verbose` - Option, ON/OFF
* `version` - Option, display/notdisplay
* `init <METHOD>` - Option, ON/OFF
* `connect` - Option, ON/OFF
* `run <Indications>` - Option, default value, (默认直接启动，否则执行 Indications 脚本)
* `poweroff=<Target>,<wait>` - Option, default value (shutdown the system)
  * `poweroff=mcu,3s`
  * `poweroff=dds,10s`
  * `poweroff=mcu`  (default value: immediately)
<!-- * `pause` -   (pause the DDS output) -->
* `monitor -p <PORT> -b <BAUD_RATE>` (参考idf.py) 
  > draw panel
  > 输入freq, 输入amp, 实际输出freq, 实际输出amp,..., ADC, DAC info, wifi, bluetooth info
  > 实时反馈
* `repl` - Option ON/OFF and **do nothing on other options**
  * 
* ~~`menuconfig` (HARD to implement)~~


步骤:

1. frontend 初始化, 指定连接方式: 蓝牙/wifi/有线(defualt)
    1. 内部调用LuatOS进行检查, ok 则 short, err 则 report

      1. PC和esp32之间建立了合法连线，且连接方式为connected

      2. PC和esp32之间建立了合法连线，但连接方式不是connected

      3. PC和esp32建立了错误连线()，但连接方式是connected **未必有这种情况，因为我们可以SPI一条线传输控制信号**

    2. 检查
2.  

```shell
ddsc init <CONNECTION_METHOD: ble/wifi/connected>

ddsc run ./script.lua # maybe lua, 

ddsc poweroff mcu:3000 # stop the while system in 3000ms
ddsc poweroff dds:300 # shutdown the DDS module in 300ms

ddsc connect # scan and try to connect to the default  MCU,  NOTICE, you need to open MCU manually!
ddsc connect --list # scan and list all available MCUs.
ddsc connect devicename

ddsc monitor -p COM6 -b 115200 # open serial port monitor at the baud rate: 115200

ddsc repl
```

## hardware todos

