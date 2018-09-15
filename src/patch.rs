use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
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

        // Don't copy the backup Assembly-CSharp.dll, as it won't be used.
        // Also ensure only DLLs are copied, to avoid unnecessary copies.
        if file_name == "Assembly-CSharp-backup.dll" || !file_name.ends_with(".dll") {
            continue;
        }
        fs::copy(&path, dir.path().join(file_name))?;
    }

    // Copy the MonoMod files and the actual patch itself.
    // We do this afterwards, in case some older versions of MonoMod's files were already in the
    // game's directory.
    for file in PATCH_FILES.iter() {
        let mut f = File::create(dir.path().join(file.0))?;
        f.write_all(file.1)?;
        f.sync_all()?;
    }

    // Make MonoMod executable for non-Windows systems
    #[cfg(unix)]
    make_monomod_executable(&dir.path().join("MonoMod.exe"))?;

    Ok(dir)
}

#[cfg(unix)]
fn make_monomod_executable(monomod_exe_path: &Path) -> io::Result<()> {
    let mut mm_permissions = fs::metadata(monomod_exe_path)?.permissions();
    mm_permissions.set_mode(0o755);
    fs::set_permissions(monomod_exe_path, mm_permissions)?;
    Ok(())
}
