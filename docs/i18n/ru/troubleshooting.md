# Troubleshooting (Русский)

Это первичная локализация Wave 1 для быстрого поиска типовых неисправностей.

Оригинал на английском:

- [../../troubleshooting.md](../../troubleshooting.md)

## Когда использовать

- Ошибки установки и запуска
- Диагностика через `status` и `doctor`
- Минимальный recovery/rollback сценарий

## Правило

- Коды ошибок, ключи логов и команды не переводятся.
- Подробные сигнатуры отказов — в английском оригинале.
- Если onboarding завершился, но TopClaw не отвечает, проверьте шаги в таком порядке: `topclaw status` -> `topclaw status --diagnose` -> `topclaw service status` -> `topclaw channel doctor`.
- Если отсутствует provider auth или срок действия auth истёк, сначала смотрите новый раздел “Provider auth missing or expired” в английском оригинале, а не webhook pairing.
- Если неясно, нужен ли `daemon` или `service`, сначала откройте [runtime-model.md](runtime-model.md).
