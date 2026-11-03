# Journal des modifications (Changelog)

Toutes les modifications notables de **Quietus** sont documentees dans ce fichier.

## [0.5.0] - 2026-11-03

### Fonctionnalites (Lot 3)
- Ajout d'une fenetre **? Restaurer une sauvegarde ?** permettant de choisir quelle session restaurer parmi l'historique `%LOCALAPPDATA%\Quietus\backups`, de visualiser les reglages contenus dans chaque sauvegarde et de supprimer d'anciens fichiers de sauvegarde.
- Ajout d'une option dans la fenetre de confirmation pour creer automatiquement un **point de restauration systeme Windows** (`Checkpoint-Computer`) avant d'appliquer les modifications.
- Horodatage des fichiers de sauvegarde a la milliseconde (`backup-YYYYMMDD-HHMMSS-mmm.json`) pour eviter toute collision.

### Securite et robustesse (Lot 2)
- Execution de `powershell.exe` via son chemin systeme absolu (`%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe`) avec `-NoLogo` pour empecher tout detournement via le `PATH` en mode administrateur.
- Validation stricte (`tasks::validate_task_path`) de tous les chemins de taches planifiees avant interpolation PowerShell (refus par defaut des caracteres d'injection).
- Verification du code de retour et remontee explicite des erreurs lorsque Windows refuse la desactivation ou reactivation d'une tache existante (`Disable/Enable-ScheduledTask`).
- Verification stricte des types de valeurs dans `registry::read` : seuls `REG_DWORD` (exactement 4 octets) et `REG_SZ`/`REG_EXPAND_SZ` sont acceptes ; tout autre type renvoie une erreur explicite au lieu d'etre converti silencieusement en chaine vide.

### Qualite, tests et hygiene (Lot 1)
- Ajout d'une suite de tests unitaires et de securite (`tweaks`, `tasks`, `registry`, `backup` en repertoire temporaire isole).
- Ajout du workflow d'integration continue GitHub Actions (`cargo fmt --check`, `cargo clippy`, `cargo test`, `cargo audit`).
- Harmonisation typographique : remplacement de tous les tirets cadratins par le trait d'union `-` dans l'ensemble du depot.
- Creation de ce fichier `CHANGELOG.md`.

## [0.4.1] - 2026-10-15

### Performance et documentation
- Regroupement de la detection des taches planifiees en un seul appel PowerShell (`tasks::statuses`), supprimant le micro-gel de l'interface au demarrage et au rafraichissement.
- Ajout d'une section dans `README.md` expliquant l'avertissement Microsoft Defender SmartScreen sur le binaire non signe.

## [0.4.0] - 2026-10-13

### Interface utilisateur
- Regroupement des reglages par categorie dans des sections repliables.
- Ajout des boutons ? Tout cocher ? et ? Tout decocher ? par categorie.
- Ajout d'une fenetre modale de confirmation recapitulative avant toute application.

## [0.3.1] - 2026-10-13

### Correctif
- Prise en charge du nouveau nom de la tache `Microsoft Compatibility Appraiser Exp` sur Windows 11 24H2 / builds 26xxx.

## [0.3.0] - 2026-10-13

### Fonctionnalites
- Prise en charge de la desactivation et restauration des taches planifiees de telemetrie (CEIP, compatibilite, feedback SIUF) via les cmdlets PowerShell non localisees.

## [0.2.0] - 2026-10-13

### Interface utilisateur
- Passage a une interface graphique native autonome (`egui` / `eframe`) et retrait de la TUI initiale.

## [0.1.0] - 2026-10-13

### Version initiale
- Premiere version de Quietus : catalogue de reglages de telemetrie Windows 11 par le registre (`winreg`), sauvegarde automatique JSON horodatee dans `%LOCALAPPDATA%\Quietus\backups` et restauration complete.
