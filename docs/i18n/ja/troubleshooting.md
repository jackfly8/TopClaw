# トラブルシューティング（日本語）

このページは Wave 1 の初版ローカライズです。よくある障害の切り分け入口です。

英語版原文:

- [../../troubleshooting.md](../../troubleshooting.md)

## 主な用途

- インストール失敗や起動不良の対応
- `status`/`doctor` を使った段階的診断
- 最小ロールバックと復旧確認

## 運用ルール

- エラーコード・ログキー・コマンド名は英語のまま保持します。
- 詳細な障害シグネチャは英語版原文を優先します。
- onboarding 完了後に「設定は終わったのに応答しない」場合は、英語版原文の順で `topclaw status` -> `topclaw status --diagnose` -> `topclaw service status` -> `topclaw channel doctor` を確認します。
- `daemon` と `service` のどちらを使うべきか迷ったら、先に [runtime-model.md](runtime-model.md) を確認します。
