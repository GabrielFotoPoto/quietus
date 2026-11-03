# Journal des modifications (Changelog)

Toutes les modifications notables de **Quietus** sont documentees dans ce fichier.

## [Non publie]

### Qualite, tests et hygiene (Lot 1)
- Ajout d'une suite de tests unitaires et de securite (`tweaks`, `tasks`, `backup` en repertoire temporaire isole).
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
- Ajout des boutons « Tout cocher » et « Tout decocher » par categorie.
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
