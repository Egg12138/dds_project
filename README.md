# TODO:

## 重构
- [ ] 对ESP32C3和IoT编程
  - [ ] 假设传输数据为'0x123123'...
  - [ ] IoTDA-MQTT 异步处理events (ESP32C3)
  - [ ] 暂时将 `require` 模块全部放在根目录下。
- [ ]  PCB
- [ ]  参考文献和章节重写。


## 命令类型

对于多数不同的指令类型，我们都可以将命令转换成64bits数据传入到对应寄存器让AD9959读取。于是在MCU眼中，有如下指令类型：

* `init`
* `reset`
* `update` (I/O Update: refresh serial port buffer of AD9959)
* `store`, store_staged_cmds
* `clearbuf`, clear_staged_cmds (clear cmds in buffer)
* set_input (transfer instructions) 
  * `set_input`, not_staged, via spi, not staged in buffer
  * `store`, via spi, staged in buffer，这里我们需要考虑，我们需要怎么让用户传入多个指令，以及如何指定这些buffer cmds有多少条？


关于 set_input 的参数, 我们会有多种模式：

如果是 `not_staged` 模式， `param` 直接包含参数。

* 直接设置期望频率，相位，幅度
* 设置扫场
* 设置多通道


```toml
# cfg.toml

[commands]
command_name = "store"

[[commands.paras]]
cmds = [
    { command_name = "set_input", 
      paras = [
        {
          freq_hz = 1000.0,
          phase_deg = 0.0, 
          amp_mv = 50,
        }
      ]
    },

    {
      command_name = "ramp",
      paras = [
        {
          target = "freq",
          from = 0.0, 
          to = 1000.0,
        }
      ] 
    }
]
```

==**我们在这个repo的`handle_multi_cmds`分支中尝试实现之**==,但由于这种多指令可能不是很必要，而且会让指令输入很麻烦，所以在`main`分支中我们只实现单指令的。即：

```toml
[[commands]]
command_name = "set_input"
paras = { freq_hz = 100, phase_deg = 45, volt_mv = 5 }

[[commands]]
command_name = "set_input"
paras = { freq_hz = 200, phase_deg = 90, volt_mv = 10 }

[[commands]]
command_name = "scan"
paras = { target = "freq", from = 1, to = 10 }
```

---

需要注意的是，前端会将参数处理为二进制指令，所以实际上MCU接收到的命令只有：

* `not_stage`
* `store`
* `report`
* `clearbuf`
* `send_buf`
