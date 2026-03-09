# Dépannage (Français)

Cette page est une localisation initiale Wave 1 pour diagnostiquer rapidement les pannes courantes.

Source anglaise:

- [../../troubleshooting.md](../../troubleshooting.md)

## Quand l'utiliser

- Échecs d'installation ou de démarrage
- Diagnostic guidé via `status` et `doctor`
- Procédures minimales de récupération/rollback

## Règle

- Les codes d'erreur, clés de logs et commandes restent en anglais.
- Les signatures de panne détaillées sont définies en anglais.
- Si l'onboarding est terminé mais que TopClaw ne répond toujours pas, vérifiez dans cet ordre : `topclaw status` -> `topclaw status --diagnose` -> `topclaw service status` -> `topclaw channel doctor`.
- Si vous hésitez entre `daemon` et `service`, lisez d'abord [runtime-model.md](runtime-model.md).
