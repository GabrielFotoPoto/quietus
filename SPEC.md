# Specification - Quietus

## Objectif

Donner a l'utilisateur un controle clair et reversible sur la telemetrie de
Windows 11 : lister les mecanismes de remontee d'informations, expliquer chacun,
et laisser l'utilisateur decider.

## Principes directeurs

1. Ne jamais casser l'OS - reglages documentes uniquement.
2. Reversibilite totale - sauvegarde avant modification, restauration possible.
3. Transparence - description, impact et risque pour chaque reglage.
4. Consentement eclaire - aucune action sans choix explicite.
5. Performance preservee.

## Architecture

- **Langage / cible** : Rust, `x86_64-pc-windows-gnu` (executable Windows natif).
- **Privileges** : manifeste `requireAdministrator` embarque (UAC).
- **Registre** : crate `winreg`.
- **UI** : interface graphique `egui` / `eframe` (executable autonome, sans
  runtime externe). La TUI initiale a ete retiree au profit du graphique.
- **Sauvegarde** : etat anterieur de chaque valeur en JSON horodate sous
  `%LOCALAPPDATA%\Quietus\backups` + point de restauration systeme Windows optionnel.

### Modele de donnees

Un `Tweak` = id, nom, categorie, risque, description, impact, redemarrage requis,
et une action (`Registry` ou `ScheduledTasks`) posant l'etat de desactivation.
La detection d'etat, l'application et l'annulation en decoulent.

## Perimetre

- **v0.1** : reglages Sur + Modere, tout par le registre (y compris type de
  demarrage des services), 100 % reversible - interface terminal (retiree).
- **v0.2** : interface graphique (egui / eframe).
- **v0.3** : taches planifiees (CEIP, compatibilite, feedback) via les cmdlets
  `Get/Disable/Enable-ScheduledTask` (sortie non localisee).
- **v0.4** : regroupement par categorie (sections repliables), ? tout cocher /
  decocher ? par categorie, fenetre de confirmation recapitulative avant application.
- **v0.4.1** : detection des taches planifiees regroupee en **un seul** appel
  PowerShell (au lieu d'un lancement de processus par tache), pour supprimer le
  micro-gel de l'interface a l'ouverture et au rafraichissement.
- **v0.5** : durcissement de securite PowerShell/registre, suite de tests et CI,
  creation optionnelle d'un point de restauration systeme Windows avant application,
  et fenetre de choix/suppression des sessions de sauvegarde a restaurer.
- **Hors perimetre** : blocage reseau des domaines Microsoft (risque de
  perturber Windows Update / Store) - ecarte par choix.

## Limites assumees

Sur edition Pro, `AllowTelemetry` ne descend pas officiellement a ? Off ? ; la
desactivation du service DiagTrack complete la mesure. L'outil l'indique.
