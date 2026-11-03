# Quietus

Gestionnaire de telemetrie **Windows 11**, controle par l'utilisateur, en Rust.

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

## Developpement

Ce projet a ete developpe avec l'assistance de l'intelligence artificielle
**Claude** (Anthropic) et **Gemini** (Google DeepMind) : specifications, conception
et generation du code Rust. Les choix de conception, la validation et les tests
restent sous la responsabilite de l'auteur du depot.

## Licence

MIT.
