# Modèle de runtime TopClaw (Français)

Cette page passerelle explique rapidement la différence entre `topclaw agent`, `service`, `daemon`, `gateway` et `channel start`.

Source anglaise:

- [../../runtime-model.md](../../runtime-model.md)

## Quand l'utiliser

- l'onboarding vient de se terminer et la prochaine commande n'est pas claire
- des canaux comme Telegram ou Discord sont configurés mais vous ne savez pas si le runtime tourne en arrière-plan
- vous voulez distinguer le chemin normal d'exploitation du chemin de debug

## Résumé rapide

- `topclaw agent` : parler directement dans ce terminal
- `topclaw service ...` : garder les canaux configurés actifs en arrière-plan
- `topclaw daemon` : déboguer le runtime complet au premier plan
- `topclaw gateway` : lancer seulement la surface HTTP/webhook
- `topclaw channel start` : lancer manuellement les canaux au premier plan, surtout pour le dépannage

## Conseils pratiques

- Pour l'exploitation normale, commencez par `topclaw service status`
- Si l'onboarding a déjà installé et démarré le service, ne lancez pas `topclaw daemon` en parallèle
- Si `--prefer-prebuilt` n'utilise pas vos changements locaux, utilisez `./bootstrap.sh --force-source-build`
