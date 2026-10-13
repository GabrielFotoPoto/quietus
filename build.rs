// Embarque un manifeste Windows demandant l'elevation administrateur (UAC).
// Sans droits admin, la plupart des cles HKLM ne sont pas modifiables.
use embed_manifest::manifest::ExecutionLevel;
use embed_manifest::{embed_manifest, new_manifest};

fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        embed_manifest(
            new_manifest("Quietus").requested_execution_level(ExecutionLevel::RequireAdministrator),
        )
        .expect("echec de l'integration du manifeste");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
