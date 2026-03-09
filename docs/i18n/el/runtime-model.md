# Μοντέλο Runtime του TopClaw

Χρησιμοποιήστε αυτή τη σελίδα όταν δεν είναι σαφές αν χρειάζεστε `topclaw agent`, `service`, `daemon`, `gateway` ή `channel start`.

Τελευταία ενημέρωση: **9 Μαρτίου 2026**.

## Σύντομη περίληψη

| Στόχος | Εντολή | Πότε χρησιμοποιείται |
|---|---|---|
| Άμεση συνομιλία στο τρέχον terminal | `topclaw agent` | γρήγορα tests και one-off prompts |
| Μόνιμη λειτουργία καναλιών στο παρασκήνιο | `topclaw service install`, `topclaw service start`, `topclaw service status` | Telegram, Discord και άλλα always-on channels |
| Πλήρες runtime στο προσκήνιο | `topclaw daemon` | debugging εκκίνησης και παρακολούθηση logs |
| Μόνο HTTP/webhook gateway | `topclaw gateway` | δοκιμές webhook |
| Χειροκίνητη εκκίνηση καναλιών στο προσκήνιο | `topclaw channel start` | προχωρημένο troubleshooting |

## Τι να περιμένετε μετά το onboarding

- ο provider πρέπει να είναι έτοιμος ή να εμφανίζεται μία σαφής εντολή auth
- το προεπιλεγμένο model πρέπει να έχει ήδη επιλεγεί
- τα επιλεγμένα channels πρέπει να έχουν αποθηκευτεί στο `config.toml`
- όταν η πλατφόρμα το υποστηρίζει, το onboarding προσπαθεί να εγκαταστήσει και να ξεκινήσει το service αυτόματα

## Ποια εντολή χρειάζομαι;

### Θέλω να δοκιμάσω γρήγορα τον assistant

```bash
topclaw agent -m "Hello, TopClaw!"
```

### Έχω ρυθμίσει background channels

```bash
topclaw service status
```

Αν δεν εκτελείται ακόμη:

```bash
topclaw service install
topclaw service start
```

### Θέλω foreground debugging

```bash
topclaw daemon
```

### Θέλω χειροκίνητο foreground channel runtime

```bash
topclaw channel doctor
topclaw channel start
```

Για κανονική λειτουργία, προτιμήστε `topclaw service ...`.
