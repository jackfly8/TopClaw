# 运行模式说明（简体中文）

这是增强型 bridge 页面，用于解释 `topclaw agent`、`service`、`daemon`、`gateway`、`channel start` 的区别。

英文原文：

- [../../runtime-model.md](../../runtime-model.md)

## 适用场景

- onboarding 完成后，不确定下一步该运行什么
- 已配置 Telegram/Discord 等通道，但不确定后台是否真的在运行
- 想区分调试命令和正常常驻命令

## 快速结论

- `topclaw agent`：在当前终端直接对话
- `topclaw service ...`：让已配置通道在后台常驻
- `topclaw daemon`：前台调试完整 runtime
- `topclaw gateway`：只启动 HTTP/webhook 网关
- `topclaw channel start`：手动前台启动通道，主要用于排障

## 使用建议

- 正常长期运行优先看 `topclaw service status`
- 如果 onboarding 已自动安装并启动服务，不需要再额外运行 `topclaw daemon`
- `systemd --user`、`launchd`、Windows 托管运行环境更可能自动完成服务接管；OpenRC 等环境仍可能需要手动执行 `topclaw service install` / `start`
- 若 `--prefer-prebuilt` 未体现本地源码改动，请改用 `./bootstrap.sh --force-source-build`
