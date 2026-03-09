# Runbook d'exploitation (Français)

Cette page est une localisation initiale Wave 1 pour les procédures day-2.

Source anglaise:

- [../../operations-runbook.md](../../operations-runbook.md)

## Quand l'utiliser

- Vérifications de santé avant/après changement
- Redémarrage des services et rollback
- Réponse standard aux incidents opérationnels

## Règle

- Les commandes et clés système restent en anglais.
- Les exigences d'exploitation finales sont définies en anglais.
- Si l'onboarding a déjà installé et démarré le service, commencez par `topclaw service status` et n'exécutez pas `topclaw daemon` en parallèle.
- Pour le chemin "résumé puis diagnostic", utilisez `topclaw status --diagnose`.
- La différence entre `agent`, `service`, `daemon` et `channel start` est résumée dans [runtime-model.md](runtime-model.md).
