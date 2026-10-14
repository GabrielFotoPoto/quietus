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
- **UI** : interface graphique `egui` / `eframe` (exécutable autonome, sans
  runtime externe). La TUI initiale a été retirée au profit du graphique.
- **Sauvegarde** : état antérieur de chaque valeur en JSON horodaté sous
  `%LOCALAPPDATA%\Quietus\backups`.

### Modèle de données

Un `Tweak` = id, nom, catégorie, risque, description, impact, redémarrage requis,
et une liste d'opérations de registre (`RegOp`) posant la valeur de désactivation.
La détection d'état, l'application et l'annulation en découlent.

## Périmètre

- **v0.1** : réglages Sûr + Modéré, tout par le registre (y compris type de
  démarrage des services), 100 % réversible — interface terminal (retirée).
- **v0.2** : interface graphique (egui / eframe).
- **v0.3** : tâches planifiées (CEIP, compatibilité, feedback) via les cmdlets
  `Get/Disable/Enable-ScheduledTask` (sortie non localisée).
- **v0.4** : regroupement par catégorie (sections repliables), « tout cocher /
  décocher » par catégorie, fenêtre de confirmation récapitulative avant application.
- **v0.4.1** : détection des tâches planifiées regroupée en **un seul** appel
  PowerShell (au lieu d'un lancement de processus par tâche), pour supprimer le
  micro-gel de l'interface à l'ouverture et au rafraîchissement.
- **v2 (à venir)** : point de restauration système, profils.
- **Hors périmètre** : blocage réseau des domaines Microsoft (risque de
  perturber Windows Update / Store) — écarté par choix.

## Limites assumées

Sur édition Pro, `AllowTelemetry` ne descend pas officiellement à « Off » ; la
désactivation du service DiagTrack complète la mesure. L'outil l'indique.
