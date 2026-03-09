# ランタイムモデル（日本語）

この強化ブリッジは、`topclaw agent`、`service`、`daemon`、`gateway`、`channel start` の役割の違いを素早く把握するためのページです。

英語版原文:

- [../../runtime-model.md](../../runtime-model.md)

## 使う場面

- onboarding 完了後に次のコマンドが分からない
- Telegram や Discord を設定したが、バックグラウンド実行が動いているか不明
- 通常運用コマンドとデバッグ用コマンドを切り分けたい

## 要点

- `topclaw agent`: この端末で直接会話
- `topclaw service ...`: 設定済みチャネルをバックグラウンドで常駐
- `topclaw daemon`: フォアグラウンドで runtime 全体をデバッグ
- `topclaw gateway`: HTTP/webhook gateway のみ起動
- `topclaw channel start`: 手動のフォアグラウンド channel 起動。主にトラブルシュート向け

## 実行ヒント

- 通常の常駐運用では、まず `topclaw service status` を確認します
- onboarding がすでに service を導入・起動しているなら、追加で `topclaw daemon` を実行する必要はありません
- `systemd --user`、`launchd`、Windows の管理ランタイムでは自動 service 化されやすく、OpenRC などでは手動で `topclaw service install` / `start` が必要な場合があります
- `--prefer-prebuilt` でローカル変更が反映されない場合は `./bootstrap.sh --force-source-build` を使います
