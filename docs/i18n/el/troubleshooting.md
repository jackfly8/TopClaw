# Αντιμετώπιση Προβλημάτων (Troubleshooting)

Αυτός ο οδηγός περιγράφει λύσεις για κοινά ζητήματα εγκατάστασης και εκτέλεσης του TopClaw.

Τελευταία ενημέρωση: 9 Μαρτίου 2026.

## Γρήγορη Εκκίνηση Διάγνωσης

| Αν συμβαίνει αυτό | Ξεκινήστε εδώ |
|---|---|
| Δεν βρίσκεται η εντολή `topclaw` μετά την εγκατάσταση | [Η εντολή `topclaw` δεν βρίσκεται μετά την εγκατάσταση](#η-εντολή-topclaw-δεν-βρίσκεται-μετά-την-εγκατάσταση) |
| Το onboarding τελείωσε αλλά το TopClaw δεν απαντά | [Το onboarding τελείωσε αλλά το TopClaw δεν απαντά](#το-onboarding-τελείωσε-αλλά-το-topclaw-δεν-απαντά) |
| Τα channels έχουν ρυθμιστεί αλλά το background runtime δεν τρέχει | [Η υπηρεσία δεν εκκινεί](#η-υπηρεσία-δεν-εκκινεί) |

---

## 1. Εγκατάσταση και Προετοιμασία (Bootstrap)

### Σφάλμα: `cargo is not installed`

**Αιτία**: Η Rust toolchain δεν είναι εγκατεστημένη.
**Λύση**:
Εκτελέστε την αυτόματη εγκατάσταση:
```bash
./bootstrap.sh --install-rust
```
Εναλλακτικά, επισκεφθείτε τη διεύθυνση [rustup.rs](https://rustup.rs/).

### Σφάλματα Μεταγλώττισης (Compilation Errors)

**Σύμπτωμα**: Αποτυχία λόγω προβλημάτων στον μεταγλωττιστή ή στο `pkg-config`.
**Λύση**:
Εγκαταστήστε τις εξαρτήσεις συστήματος (system dependencies):
```bash
./bootstrap.sh --install-system-deps
```

### Η εντολή `topclaw` δεν βρίσκεται μετά την εγκατάσταση

```bash
export PATH="$HOME/.cargo/bin:$PATH"
which topclaw
```

Αν χρειάζεται, προσθέστε το στο shell profile σας.

### Περιορισμένοι Πόροι (RAM / Disk Space)

**Σύμπτωμα**: Τερματισμός της διαδικασίας από τον OOM killer ή σφάλμα `cannot allocate memory`.
**Λύση**:
Χρησιμοποιήστε προ-μεταγλωττισμένα (prebuilt) αρχεία:
```bash
./bootstrap.sh --prefer-prebuilt
```
Εάν επιθυμείτε μεταγλώττιση από τον πηγαίο κώδικα σε περιβάλλον με περιορισμένη μνήμη, περιορίστε τον παραλληλισμό:
```bash
CARGO_BUILD_JOBS=1 cargo build --release --locked
```

---

## 2. Χρόνος Εκκίνησης και Συνδεσιμότητα (Runtime)

### Αργή Μεταγλώττιση

Η στοίβα Matrix E2EE και τα native scripts για κρυπτογραφία απαιτούν σημαντικούς πόρους.
- Για ταχύτερο τοπικό έλεγχο χωρίς Matrix:
  ```bash
  cargo check
  ```
- Για ανάλυση χρόνων μεταγλώττισης:
  ```bash
  cargo check --timings
  ```

### Η Πύλη (Gateway) δεν είναι προσβάσιμη

Επαληθεύστε την κατάσταση του συστήματος:
```bash
topclaw status
topclaw doctor
```
Ελέγξτε τις ρυθμίσεις στο `~/.topclaw/config.toml`:
- `[gateway].host` (Προεπιλογή: `127.0.0.1`)
- `[gateway].port` (Προεπιλογή: `42617`)

### Το onboarding τελείωσε αλλά το TopClaw δεν απαντά

Ελέγξτε με αυτή τη σειρά:

```bash
topclaw status
topclaw status --diagnose
topclaw service status
topclaw channel doctor
```

Συνήθεις αιτίες:

- ο provider δεν έχει ολοκληρώσει authentication
- η υπηρεσία δεν έχει εγκατασταθεί ή δεν τρέχει
- λείπουν channel tokens ή allowlists
- η πλατφόρμα σας απαιτεί χειροκίνητο service setup

---

## 3. Κανάλια Επικοινωνίας (Channels)

### Σύγκρουση Telegram: `terminated by other getUpdates request`

**Αιτία**: Πολλαπλά instances χρησιμοποιούν το ίδιο Bot Token.
**Λύση**: Τερματίστε όλες τις άλλες διεργασίες που χρησιμοποιούν το συγκεκριμένο token.

### Διάγνωση Καναλιού

Εκτελέστε την εντολή:
```bash
topclaw channel doctor
```

---

## 4. Λειτουργία ως Υπηρεσία (Service Mode)

Για να παραμένει το TopClaw ενεργό στο παρασκήνιο:

```bash
topclaw service install
topclaw service start
topclaw service status
```

Για καθαρή επανεκκίνηση:

```bash
topclaw service stop
topclaw service start
```

### Η υπηρεσία δεν εκκινεί

Ελέγξτε την κατάσταση μέσω του TopClaw CLI:
```bash
topclaw service status
```
Για προβολή των logs στο Linux:
```bash
journalctl --user -u topclaw.service -f
```

---

## 5. Υποβολή Αναφοράς Προβλήματος

Εάν το πρόβλημα επιμένει, συμπεριλάβετε τα αποτελέσματα των παρακάτω εντολών στην αναφορά σας:
```bash
topclaw --version
topclaw status
topclaw doctor
topclaw channel doctor
```

> [!TIP]
> Παρακαλείστε να αφαιρέσετε τυχόν ευαίσθητα δεδομένα (API keys, Tokens) από το αρχείο ρυθμίσεων πριν το κοινοποιήσετε.
