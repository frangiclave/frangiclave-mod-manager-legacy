use patch;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::str;

const EXE_PATH: &'static str = "cultistsimulator.exe";
const MANAGED_PATH: &'static str = "cultistsimulator_Data/Managed";
const ASSEMBLY_PATH: &'static str = "cultistsimulator_Data/Managed/Assembly-CSharp.dll";
const ASSEMBLY_BACKUP_PATH: &'static str =
    "cultistsimulator_Data/Managed/Assembly-CSharp-backup.dll";
const CORE_PATH: &'static str = "cultistsimulator_Data/StreamingAssets/content/core";
const MORE_PATH: &'static str = "cultistsimulator_Data/StreamingAssets/content/more";
const MODS_PATH: &'static str = "cultistsimulator_Data/StreamingAssets/mods";

pub struct Game {
    exe_path: PathBuf,
    managed_path: PathBuf,
    assembly_path: PathBuf,
    assembly_backup_path: PathBuf,
    core_path: PathBuf,
    more_path: PathBuf,
    mods_path: PathBuf,
}

impl Game {
    pub fn new(root: &PathBuf) -> Game {
        Game {
            exe_path: root.join(EXE_PATH),
            managed_path: root.join(MANAGED_PATH),
            assembly_path: root.join(ASSEMBLY_PATH),
            assembly_backup_path: root.join(ASSEMBLY_BACKUP_PATH),
            core_path: root.join(CORE_PATH),
            more_path: root.join(MORE_PATH),
            mods_path: root.join(MODS_PATH),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.exe_path.is_file() && self.assembly_path.is_file()
    }

    pub fn patch_assembly(&self) -> Result<(), String> {
        // If no backup assembly exists, create one, then use the backup assembly as a basis for the
        // patch.
        // This is to prevent double-patching the assembly.
        if !self.assembly_backup_path.is_file() {
            match fs::copy(&self.assembly_path, &self.assembly_backup_path) {
                Ok(_) => (),
                Err(e) => {
                    return Err(format!(
                        "Failed to copy {}: {}",
                        &self.assembly_path.display(),
                        e
                    ))
                }
            }
        }
        let dir = match patch::setup_patch_directory(&self.managed_path) {
            Ok(d) => d,
            Err(e) => return Err(format!("Failed to set up temporary patch directory: {}", e)),
        };

        // Run MonoMod to patch the clean assembly.
        match Command::new(dir.path().join("MonoMod.exe"))
            .arg("Assembly-CSharp.dll")
            .current_dir(dir.path())
            .output()
        {
            Ok(output) => {
                if !output.status.success() {
                    return Err(format!(
                        "MonoMod failed to patch Assembly-CSharp.dll: {}",
                        str::from_utf8(&output.stderr).unwrap()
                    ));
                }
            }
            Err(e) => return Err(format!("Failed to start MonoMod: {}", e)),
        }

        // Prepare the patch directory for copy back to the game directory.
        fs::remove_file(dir.path().join("MONOMODDED_Assembly-CSharp.pdb")).unwrap();
        fs::remove_file(dir.path().join("Assembly-CSharp.FrangiclavePatch.mm.dll")).unwrap();
        fs::rename(
            dir.path().join("MONOMODDED_Assembly-CSharp.dll"),
            dir.path().join("Assembly-CSharp.dll"),
        ).unwrap();

        // Copy every file back, as the patched assembly will need MonoMod and its dependencies.
        for dir_entry in fs::read_dir(dir.path()).unwrap() {
            let path = &dir_entry.unwrap().path();
            let file_name = path.file_name().unwrap().to_str().unwrap();
            match fs::copy(path, self.managed_path.join(file_name)) {
                Ok(_) => (),
                Err(e) => return Err(format!("Failed to copy {}: {}", path.display(), e))
            }
        }

        dir.close().unwrap();
        Ok(())
    }

    pub fn make_mods_dir(&self) -> io::Result<()> {
        fs::create_dir_all(&self.mods_path)
    }
}
