use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use tempdir::TempDir;

const PATCH_FILES: [(&str, &[u8]); 6] = [
    (
        "Assembly-CSharp.FrangiclavePatch.mm.dll",
        include_bytes!("../data/patch/Assembly-CSharp.FrangiclavePatch.mm.dll"),
    ),
    (
        "Mono.Cecil.dll",
        include_bytes!("../data/patch/Mono.Cecil.dll"),
    ),
    (
        "Mono.Cecil.Mdb.dll",
        include_bytes!("../data/patch/Mono.Cecil.Mdb.dll"),
    ),
    (
        "Mono.Cecil.Pdb.dll",
        include_bytes!("../data/patch/Mono.Cecil.Pdb.dll"),
    ),
    ("MonoMod.exe", include_bytes!("../data/patch/MonoMod.exe")),
    (
        "MonoMod.Utils.dll",
        include_bytes!("../data/patch/MonoMod.Utils.dll"),
    ),
];

pub fn setup_patch_directory(managed_path: &Path) -> io::Result<TempDir> {
    let dir = TempDir::new("frangiclave-patch")?;

    // Copy the Cultist Simulator DLLs.
    for dir_entry in fs::read_dir(managed_path).unwrap() {
        let path = dir_entry.unwrap().path();
        let file_name = path.file_name().unwrap().to_str().unwrap();

        // Don't copy the original Assembly-CSharp.dll, as it may have already been patched and
        // should not be patched over.
        // Also ensure only DLLs are copied, to avoid unnecessary copies.
        if file_name == "Assembly-CSharp.dll" || !file_name.ends_with(".dll") {
            continue;
        }
        fs::copy(&path, dir.path().join(file_name))?;
    }

    // Use the backup as the basis for the patch.
    fs::copy(
        dir.path().join("Assembly-CSharp-backup.dll"),
        dir.path().join("Assembly-CSharp.dll"),
    )?;

    // Copy the MonoMod files and the actual patch itself.
    // We do this afterwards, in case some older versions of MonoMod's files were already in the
    // game's directory.
    for file in PATCH_FILES.iter() {
        let mut f = File::create(dir.path().join(file.0))?;
        f.write_all(file.1)?;
        f.sync_all()?;
    }

    Ok(dir)
}
