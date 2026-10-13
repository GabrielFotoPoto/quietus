# Quietus

Gestionnaire de télémétrie **Windows 11**, contrôlé par l'utilisateur, en Rust.

*Quietus* : la mise au silence du bruit de fond que votre machine émet sans vous.

L'outil **liste** les mécanismes de remontée d'informations de Windows, **explique**
pour chacun ce qu'il fait et ce que sa désactivation implique, puis laisse
l'utilisateur **choisir** ce qu'il désactive. Chaque modification est
**sauvegardée avant d'être appliquée** et **réversible**.

## Principes

- **Ne casse pas l'OS** : uniquement des réglages documentés et compris.
- **Réversibilité** : sauvegarde automatique de l'état antérieur, bouton « tout restaurer ».
- **Transparence** : description + impact + niveau de risque pour chaque réglage.
- **Consentement** : rien n'est appliqué sans action explicite de l'utilisateur.

## Utilisation

L'outil requiert les **droits administrateur** (demande d'élévation automatique) pour
les réglages au niveau machine.

```
quietus.exe
```

- `↑/↓` : naviguer   `Espace` : cocher   `a` : appliquer la sélection
- `r` : restaurer la dernière sauvegarde   `q` : quitter

Les sauvegardes sont écrites dans `%LOCALAPPDATA%\Quietus\backups`.

## Portée (v1)

Réglages de niveau **Sûr** et **Modéré**, tous réversibles, par le registre :
télémétrie cœur (AllowTelemetry, service DiagTrack, dmwappushservice), rapports
d'erreurs et feedback, identifiant de publicité et expériences personnalisées,
suggestions du menu Démarrer, historique d'activité, suivi des lancements d'apps,
personnalisation de la saisie, localisation.

> **Note d'honnêteté** : sur une édition **Pro**, la clé `AllowTelemetry` ne
> descend pas officiellement à zéro (socle « Requis » maintenu par Microsoft) ;
> la désactivation du service DiagTrack complète néanmoins fortement la mesure.

## Compilation

Binaire Windows produit par compilation croisée depuis Linux :

```
cargo build --release --target x86_64-pc-windows-gnu
```

## Avertissement

Logiciel fourni « tel quel », sans garantie. Il modifie des réglages système de
**votre propre machine**. Utilisez la fonction de restauration en cas de doute.

## Licence

MIT.
