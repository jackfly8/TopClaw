# Αντιμετώπιση Προβλημάτων TopClaw (Ελληνικά)

Αυτή είναι σελίδα-γέφυρα για κοινά προβλήματα εγκατάστασης, runtime και channel.

Αγγλικό πρωτότυπο:

- [../../troubleshooting.md](../../troubleshooting.md)

## Πότε να τη χρησιμοποιήσετε

- Όταν το TopClaw δεν απαντά μετά το onboarding
- Όταν λείπει ή έχει λήξει το provider auth
- Όταν το background service ή τα channels δεν λειτουργούν σωστά

## Κανόνας

- Οι κωδικοί σφαλμάτων, τα log keys και οι ακριβείς failure signatures ορίζονται στην αγγλική τεκμηρίωση.
- Για ασφαλή διάγνωση, προχωρήστε με:
  `topclaw status` -> `topclaw status --diagnose` -> `topclaw service status` -> `topclaw channel doctor`
