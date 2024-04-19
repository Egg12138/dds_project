# today


## project software demo todos

TODO: rust net tcp socket
TODO: luatos tcp socket
TODO: PCB board
TODO: Manual & notes -> paper

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
##### notes

luatos DDS-DEMO (1h) 
尝试用string(不行就改用zbuff)
- [x]  -> luatos refactor (0.5h)
    -> LEARN ad9959 manual
      -> LEARN ad9959_python, 看看要不要换Arduino
        -> Frontend transfer API refactor
      LEARN 仿照python用lua来实现

TODO Rust Aciton，socket
LEARN luatos LVGL learning basic: (4h)
* install SquareLine Stuidio
* install simulator (SDL2 + VS?)

* tokio


**重要重构——妈的，失算！**

`CommandTypes`应该为:

```rust
enum CommandTypes {
  SetInput(InputParas),
  ...
}
```

在cfg中为：

```toml
command_names = { "input" = {freq_hv = 3,14, volt_mv = 6666, phs_oft = 90}}
```


### communcation

via SPI protocol


#### WIFI-mangement

```lua
-- 中文注释由LLM生成

--[=====[
--- WiFi Manager模块提供了一套全面的函数，用于在ESP32C3设备上管理Wi-Fi连接和配置。

-- @module wifi-manager

-- @field [readonly] string HOST_NAME 设备分配的主机名。
-- @field [readonly] string BUILTIN_SSID 内置Wi-Fi网络的默认SSID。
-- @field [readonly] sring BUILTIN_PWD 内置Wi-Fi网络的默认密码。
-- @field [readonly] string SSID_AS_AP 设备作为接入点（AP）时使用的SSID。
-- @field [readonly] string PWD_AS_AP 设备作为接入点（AP）时使用的密码。

-- @type ManagerObject
-- @table manager
-- `manager`对象包含了管理Wi-Fi连接与配置的各种方法。

-- @function [parent=ManagerObject] dbg
-- @tparam table t 要打印的表。
-- @brief 一个调试工具函数，用于打印给定表`t`的内容。

-- @function [parent=ManagerObject] save
-- @brief 保存`manager.wifilist`中存储的Wi-Fi网络到持久化存储（使用`fskv`）。
-- @see ManagerObject:wifilist

-- @function [parent=ManagerObject] load
-- @brief 从持久化存储（`fskv`）中加载Wi-Fi网络到`manager.wifilist`。
-- @see ManagerObject:wifilist

-- @function [parent=ManagerObject] remove_fromfskv
-- @tparam string ssid 要移除的Wi-Fi网络的SSID。
-- @brief 从持久化存储（`fskv`）中移除指定`ssid`的Wi-Fi配置。

-- @function [parent=ManagerObject] add
-- @tparam string ssid Wi-Fi网络的SSID。
-- @tparam string pwd Wi-Fi网络的密码。
-- @brief 向`manager.wifilist`添加一个新的Wi-Fi网络。

-- @function [parent=ManagerObject] getpwd
-- @tparam string ssid Wi-Fi网络的SSID。
-- @treturn string pwd 如果找到，返回Wi-Fi网络的密码，否则返回`nil`。
-- @brief 从`manager.wifilist`中检索与指定`ssid`关联的密码。

-- @function [parent=ManagerObject] has
-- @tparam string ssid Wi-Fi网络的SSID。
-- @treturn boolean 如果Wi-Fi网络存在，返回`true`，否则返回`false`。
-- @brief 检查`manager.wifilist`中是否存在指定`ssid`的Wi-Fi网络。

-- @function [parent=ManagerObject] isdiscoer
-- @tparam string ssid Wi-Fi网络的SSID。
-- @treturn boolean 如果网络已被发现，返回`true`，否则返回`false`，或如果网络不在`manager.wifilist`中，返回`nil`。
-- @brief 判断具有指定`ssid`的Wi-Fi网络是否已被发现。

-- @function [parent=ManagerObject] found
-- @tparam string ssid Wi-Fi网络的SSID。
-- @brief 标记具有指定`ssid`的Wi-Fi网络为已发现。

-- @function [parent=ManagerObject] scan_and_connect
-- @tparam enum mode Wi-Fi模式（wlan.STATION或wlan.AP）。
-- @tparam number timeout (可选) 连接重试的超时时间（以毫秒为单位，默认：1200）。
-- @brief 根据指定的`mode`（STATION/AP）扫描Wi-Fi网络并尝试连接，可选设置`timeout`。

-- @function [parent=ManagerObject] init
-- @brief 初始化Wi-Fi模块，设置主机名，并将默认内置Wi-Fi网络添加到`manager.wifilist`。

-- @function [parent=ManagerObject] scan
-- @brief 启动Wi-Fi扫描。

-- @function [parent=ManagerObject] pairs
-- @treturn function 用于遍历`manager.wifilist`中Wi-Fi网络的迭代器函数。
-- @brief 返回一个用于遍历`manager.wifilist`中Wi-Fi网络的迭代器函数。

-- @function fskv_seT_last
-- @tparam string ssid Wi-Fi网络的SSID。
-- @tparam string pwd Wi-Fi网络的密码。
-- @brief 使用`fskv`将最近连接的Wi-Fi网络的`ssid`和`pwd`保存到持久化存储。

-- @function [parent=ManagerObject] connect
-- @brief 尝试连接到最近连接的Wi-Fi网络，如果失败，则尝试连接`manager.wifilist`中任何已知的网络。

-- @function [parent=ManagerObject] disconnect
-- @brief 断开当前连接的Wi-Fi网络。

-- @function [parent=ManagerObject] simpleRun
-- @brief Wi-Fi Manager的简化初始化序列：初始化、加载配置、扫描、连接、保存电源设置，并记录成功信息。

-- @return 包含上述方法和属性的`manager`对象。
--]=====]




```

## hardware todos



