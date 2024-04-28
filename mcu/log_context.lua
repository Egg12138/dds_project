local logs = {}

logs.enlog = {}
logs.zhlog = {}

-- Errors

logs.enlog.unsupported = "Unsupported"
logs.zhlog.unsupported = "不支持"

logs.enlog.unsupported_mcu = "Unsupported board, only ESP32C3(luatos) and ESP32S3(luatos) are supported"
logs.zhlog.unsupported_mcu = "不支持的MCU, 目前仅支持ESP32C3(luatos)和ESP32S3(luatos)"

logs.enlog.invalid_command = "Invalid command, please check the command name"
logs.zhlog.invalid_command = "无效的命令, 请检查命令名称"

logs.enlog.invalid_paras = "Invalid parameters, please check the parameters"
logs.zhlog.invalid_paras = "无效的参数, 请检查参数"

logs.enlog.spi_creat_err = "create SPI device failed"
logs.zhlog.spi_creat_err = "创建SPI设备抽象失败"

-- Report

logs.enlog.re = "Retry"
logs.zhlog.re = "重试"

logs.enlog.fin = "Finished"
logs.zhlog.fin = "完成"

logs.enlog.begin = "Start"
logs.zhlog.begin = "开始"

logs.enlog.spi_creat_ok = "create SPI device success"
logs.zhlog.spi_creat_ok = "创建SPI设备抽象成功"

logs.enlog.spi_trans_ok = "SPI Transfer successfully"
logs.zhlog.spi_trans_ok = "SPI传输成功"

logs.enlog.communicate_socket = "Cummincate with host via socket"
logs.zhlog.communicate_socket = "通过socket与主机通信"

logs.enlog.communicate_iot = "Commincate with host via MQTT IoT platform"
logs.zhlog.communicate_iot = "通过MQTT物联网平台与主机通信"


return logs