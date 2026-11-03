<p align="center">
  <img src="docs/logo/quietus.svg" alt="Logo Quietus" width="140" height="140">
</p>

<h1 align="center">Quietus</h1>

<p align="center">
  Gestionnaire de telemetrie <b>Windows 11</b>, controle par l'utilisateur et <b>reversible</b>
</p>

<p align="center">
  <a href="https://github.com/GabrielFotoPoto/quietus/actions/workflows/ci.yml"><img src="https://github.com/GabrielFotoPoto/quietus/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/GabrielFotoPoto/quietus/releases/latest"><img src="https://img.shields.io/github/v/release/GabrielFotoPoto/quietus?color=1d428a&label=version" alt="Derniere version"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/licence-MIT-d86018" alt="Licence MIT"></a>
  <img src="https://img.shields.io/badge/plateforme-Windows%2011%20(x64)-1d428a" alt="Plateforme Windows 11">
  <img src="https://img.shields.io/badge/approche-vibe%20coding%20(Claude%20%2B%20Gemini)-50b450" alt="Vibe coding Claude + Gemini">
</p>

<p align="center">
  <a href="#utilisation">Utilisation</a> &middot;
  <a href="#au-premier-lancement-smartscreen">SmartScreen</a> &middot;
  <a href="#developpement-vibe-coding">Vibe coding</a> &middot;
  <a href="CHANGELOG.md">Journal des modifications</a> &middot;
  <a href="SPEC.md">Specification</a>
</p>

---

*Quietus* : la mise au silence du bruit de fond que votre machine emet sans vous.

L'outil **liste** les mecanismes de remontee d'informations de Windows, **explique**
pour chacun ce qu'il fait et ce que sa desactivation implique, puis laisse
l'utilisateur **choisir** ce qu'il desactive. Chaque modification est
**sauvegardee avant d'etre appliquee** et **reversible**.

## Principes

- **Ne casse pas l'OS** : uniquement des reglages documentes et compris.
- **Reversibilite** : sauvegarde automatique de l'etat anterieur (JSON local + point de restauration systeme Windows optionnel), fenetre **? Restaurer... ?** permettant de choisir quelle session restaurer.
- **Transparence** : description + impact + niveau de risque pour chaque reglage.
- **Consentement** : rien n'est applique sans action explicite de l'utilisateur.

## Utilisation

L'outil requiert les **droits administrateur** (demande d'elevation automatique) pour
les reglages au niveau machine.

```
quietus.exe
```

Une fenetre s'ouvre : les reglages sont regroupes par categorie dans des sections
repliables (chacune avec un etat et des boutons **Tout cocher / decocher**), et le
detail du reglage selectionne s'affiche a droite. Cochez ce que vous voulez
desactiver puis cliquez sur **Appliquer la selection** : une fenetre recapitule la
liste avant confirmation (avec option de creer un point de restauration systeme
Windows). Le bouton **Restaurer...** permet de choisir une sauvegarde anterieure
pour remettre ses reglages a leur etat initial ou de supprimer d'anciennes
sauvegardes.

Les sauvegardes sont ecrites dans `%LOCALAPPDATA%\Quietus\backups`.

## Au premier lancement (SmartScreen)

Le binaire n'est **pas signe numeriquement**. Au premier lancement apres
telechargement, Microsoft Defender **SmartScreen** affiche un avertissement
? Windows a protege votre PC / Editeur inconnu ?. C'est normal pour un outil
open-source non signe ; le code source est public dans ce depot.

Deux facons de passer l'avertissement :

- **Debloquer le fichier** (recommande) : clic droit sur `quietus.exe` ->
  **Proprietes** -> cocher **? Debloquer ?** -> **OK**. Ou en PowerShell :
  ```powershell
  Unblock-File .\quietus.exe
  ```
- **Executer quand meme** : sur l'ecran SmartScreen, cliquer sur
  **? Informations complementaires ?** puis **? Executer quand meme ?**.

L'invite **administrateur** (UAC) jaune qui suit est, elle, **voulue** : l'outil
a besoin des droits machine pour certains reglages.

## Portee

Reglages de niveau **Sur** et **Modere**, tous reversibles, par le registre ou les
taches planifiees :
telemetrie coeur (AllowTelemetry, service DiagTrack, dmwappushservice, taches CEIP
et compatibilite), rapports d'erreurs et feedback, identifiant de publicite et
experiences personnalisees, suggestions du menu Demarrer, historique d'activite,
suivi des lancements d'apps, personnalisation de la saisie, localisation.

> **Note d'honnetete** : sur une edition **Pro**, la cle `AllowTelemetry` ne
> descend pas officiellement a zero (socle ? Requis ? maintenu par Microsoft) ;
> la desactivation du service DiagTrack complete neanmoins fortement la mesure.

## Compilation et tests

Binaire Windows produit par compilation croisee depuis Linux :

```bash
cargo test
cargo clippy --target x86_64-pc-windows-gnu --all-targets -- -D warnings
cargo build --release --target x86_64-pc-windows-gnu
```

## Avertissement

Logiciel fourni ? tel quel ?, sans garantie. Il modifie des reglages systeme de
**votre propre machine**. Utilisez la fonction de restauration en cas de doute.

## Developpement (Vibe coding)

Par transparence, ce projet est integralement realise en **vibe coding** avec
l'assistance des modeles d'intelligence artificielle **Claude** (Anthropic) et
**Gemini** (Google DeepMind) : redaction des specifications, conception de
l'architecture, generation du code Rust, des tests unitaires et de l'integration
continue.

Le cadrage fonctionnel, les exigences de securite (*Security by design*) et de
confidentialite (*Privacy by design*), la revue de chaque lot et la validation en
conditions reelles sur Windows 11 sont assures par l'auteur du depot.

## Licence

MIT.
