# Модель рантайма TopClaw (Русский)

Это bridge-страница для быстрого понимания различий между `topclaw agent`, `service`, `daemon`, `gateway` и `channel start`.

Английский оригинал:

- [../../runtime-model.md](../../runtime-model.md)

## Когда читать

- после onboarding непонятно, какую команду запускать дальше
- настроены Telegram/Discord-каналы, но неясно, работает ли фоновой runtime
- нужно отделить обычный путь эксплуатации от режима отладки

## Коротко

- `topclaw agent`: прямой диалог в текущем терминале
- `topclaw service ...`: постоянная фоновая работа настроенных каналов
- `topclaw daemon`: отладка полного runtime в foreground
- `topclaw gateway`: запуск только HTTP/webhook gateway
- `topclaw channel start`: ручной foreground-запуск каналов для advanced troubleshooting

## Практические подсказки

- Для обычной постоянной работы сначала проверьте `topclaw service status`
- Если onboarding уже установил и запустил service, не нужно параллельно запускать `topclaw daemon`
- В средах с `systemd --user`, `launchd` и управляемым Windows runtime service чаще запускается автоматически; в OpenRC и похожих окружениях `topclaw service install` / `start` может понадобиться вручную
- Если `--prefer-prebuilt` не показывает локальные изменения исходников, используйте `./bootstrap.sh --force-source-build`
