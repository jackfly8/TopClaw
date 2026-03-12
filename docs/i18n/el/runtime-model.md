# Μοντέλο Runtime TopClaw (Ελληνικά)

Αυτή είναι σελίδα-γέφυρα για να επιλέξετε ανάμεσα σε `agent`, `service`, `daemon` και `gateway`.

Αγγλικό πρωτότυπο:

- [../../runtime-model.md](../../runtime-model.md)

## Πότε να τη χρησιμοποιήσετε

- Όταν δεν είναι σαφές αν χρειάζεται άμεσο terminal chat ή background service
- Όταν θέλετε να καταλάβετε πότε το `daemon` είναι προτιμότερο από το `service`
- Όταν χρειάζεται να διακρίνετε τη ροή μεταξύ CLI, channels και gateway

## Κανόνας

- Η ακριβής runtime συμπεριφορά και τα decision paths ορίζονται στην αγγλική τεκμηρίωση.
- Για always-on channels όπως Telegram και Discord, δώστε προτεραιότητα στην αγγλική ενότητα `service`.
