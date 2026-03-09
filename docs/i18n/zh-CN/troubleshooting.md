# 故障排查（简体中文）

这是 Wave 1 首版本地化页面，用于快速定位常见故障与恢复路径。

英文原文：

- [../../troubleshooting.md](../../troubleshooting.md)

## 适用场景

- 安装失败、运行异常、通道故障
- 结合 `status`/`doctor` 做分层诊断
- 执行最小化回滚与恢复验证

## 使用建议

- 错误码、日志字段与命令名保持英文。
- 详细故障签名以英文原文为准。
- onboarding 完成后如果“看起来配置好了但仍然不回复”，优先按英文原文中的顺序检查：`topclaw status` -> `topclaw status --diagnose` -> `topclaw service status` -> `topclaw channel doctor`。
- 若不确定该启动 `daemon` 还是 `service`，先看 [runtime-model.md](runtime-model.md)。
