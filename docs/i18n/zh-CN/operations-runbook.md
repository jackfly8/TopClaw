# 运维 Runbook（简体中文）

这是 Wave 1 首版本地化页面，用于日常运维（Day-2）流程导航。

英文原文：

- [../../operations-runbook.md](../../operations-runbook.md)

## 适用场景

- 变更前后健康检查
- 服务重启、回滚与稳定性验证
- 运维异常的标准处置路径

## 使用建议

- 运维命令与系统键名保持英文。
- 生产操作步骤以英文原文为准。
- onboarding 如果已经自动安装并启动后台服务，先运行 `topclaw service status`，不要再额外并行启动 `topclaw daemon`。
- 如果想先看摘要再进入诊断，优先使用 `topclaw status --diagnose`。
- 关于 `agent` / `service` / `daemon` / `channel start` 的角色差异，参见 [runtime-model.md](runtime-model.md)。
