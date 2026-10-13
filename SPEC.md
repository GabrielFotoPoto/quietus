# Spécification — Quietus

## Objectif

Donner à l'utilisateur un contrôle clair et réversible sur la télémétrie de
Windows 11 : lister les mécanismes de remontée d'informations, expliquer chacun,
et laisser l'utilisateur décider.

## Principes directeurs

1. Ne jamais casser l'OS — réglages documentés uniquement.
2. Réversibilité totale — sauvegarde avant modification, restauration possible.
3. Transparence — description, impact et risque pour chaque réglage.
4. Consentement éclairé — aucune action sans choix explicite.
5. Performance préservée.

## Architecture

- **Langage / cible** : Rust, `x86_64-pc-windows-gnu` (exécutable Windows natif).
- **Privilèges** : manifeste `requireAdministrator` embarqué (UAC).
- **Registre** : crate `winreg`.
- **UI** : TUI `ratatui` + `crossterm` (v1). GUI `egui` envisagée en v2.
- **Sauvegarde** : état antérieur de chaque valeur en JSON horodaté sous
  `%LOCALAPPDATA%\Quietus\backups`.

### Modèle de données

Un `Tweak` = id, nom, catégorie, risque, description, impact, redémarrage requis,
et une liste d'opérations de registre (`RegOp`) posant la valeur de désactivation.
La détection d'état, l'application et l'annulation en découlent.

## Périmètre

- **v1** : réglages Sûr + Modéré, tout par le registre (y compris type de
  démarrage des services), 100 % réversible.
- **v1.1** : tâches planifiées (CEIP) via Planificateur de tâches.
- **v2** : interface graphique (egui), point de restauration système, profils.
- **Hors périmètre v1** : blocage réseau des domaines Microsoft (risque de
  perturber Windows Update / Store) — écarté par choix.

## Limites assumées

Sur édition Pro, `AllowTelemetry` ne descend pas officiellement à « Off » ; la
désactivation du service DiagTrack complète la mesure. L'outil l'indique.
