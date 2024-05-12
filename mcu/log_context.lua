local logs = {}

logs.enlog = {}
logs.zhlog = {}

-- Source
logs.enlog.usage = "The payload from IoT platform contains CommandType && Command by 64 bits"
logs.zhlog.usage = "IoT平台下发的命令包含 CommandType && 64位指令字"

logs.enlog.shutdown = "Shutdown."
logs.zhlog.shutdown = "关闭系统"

-- Errors

logs.enlog.unsupported = "Unsupported"
logs.zhlog.unsupported = "不支持"

logs.enlog.overflow = "SPI Command buffer overflow"
logs.zhlog.overflow = "SPI 指令缓存区已满"

logs.enlog.outrange = "index out of range"
logs.zhlog.outrange = "索引越界"

logs.enlog.notready = "Not ready"
logs.zhlog.notready = "未就绪"

logs.enlog.unsupported_mcu = "Unsupported board, only ESP32C3(luatos) and ESP32S3(luatos) are supported"
logs.zhlog.unsupported_mcu = "不支持的MCU, 目前仅支持ESP32C3(luatos)和ESP32S3(luatos)"

logs.enlog.invalid_command = "Invalid command, please check the command name"
logs.zhlog.invalid_command = "无效的命令, 请检查命令名称"

logs.enlog.invalid_paras = "Invalid parameters, please check the parameters"
logs.zhlog.invalid_paras = "无效的参数, 请检查参数"

logs.enlog.not_table = "#1 argument is not a table!"
logs.zhlog.not_table = "#1参数不是table!"


logs.enlog.spi_creat_err = "create SPI device failed"
logs.zhlog.spi_creat_err = "创建SPI设备抽象失败"

logs.enlog.iot_connection_lost = "MQTT IoT platform connection lost"
logs.zhlog.iot_connection_lost = "MQTT物联网平台连接丢失"




-- Report

logs.enlog.re = "Retry"
logs.zhlog.re = "重试"

logs.enlog.fin = "Finished"
logs.zhlog.fin = "处理完成"

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

logs.enlog.iot_not_connected = "MQTT IoT platform is not connected"
logs.zhlog.iot_not_connected = "MQTT物联网平台未连接"

logs.enlog.iot_not_auth = "MQTT IoT platform hasn't been aurhorizaed"
logs.zhlog.iot_not_auth = "MQTT物联网平台还未授权"


logs.enlog.iot_conack = "MQTT: connection responsed"
logs.zhlog.iot_conack = "MQTT 响应报文"

logs.enlog.iot_authorize_ok = "MQTT IoT platform authorization success, now you can subscribe and publish data"
logs.zhlog.iot_authorize_ok = "MQTT物联网平台授权成功，现在你可以订阅和发布数据了"

logs.enlog.received_datapkg = "Received data package from IoT platform"
logs.zhlog.received_datapkg = "从物联网平台接收到数据包"

return logs