use patch;
use regex::Regex;
use semver::Version;
use serde_json;
use std::fs;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

const WINDOWS_EXE_PATH: &'static str = "cultistsimulator.exe";
const MACOS_EXE_PATH: &'static str = "Contents/MacOS/OSX";
const LINUX_EXE_PATH: &'static str = "CS.x86_64";
const WINDOWS_DATA_PATH: &'static str = "cultistsimulator_Data/";
const MACOS_DATA_PATH: &'static str = "Contents/Resources/Data/";
const LINUX_DATA_PATH: &'static str = "CS.x86_64_Data/";

const MANAGED_PATH: &'static str = "Managed/";
const ASSEMBLY_PATH: &'static str = "Managed/Assembly-CSharp.dll";
const ASSEMBLY_BACKUP_PATH: &'static str = "Managed/Assembly-CSharp-backup.dll";
const MODS_PATH: &'static str = "StreamingAssets/mods";

const MOD_DEPENDENCY_VERSION: &'static str = r"^\s*(\w+)(?:\s*(<=|<|>=|>|==)\s*([\d.]+))?\s*$";

pub struct Game {
    exe_path: PathBuf,
    managed_path: PathBuf,
    assembly_path: PathBuf,
    assembly_backup_path: PathBuf,
    mods_path: PathBuf,
}

impl Game {
    pub fn new(root: &PathBuf) -> Game {
        let exe_path;
        let data_path;
        if cfg!(all(target_os="windows", target_arch="x86_64")) {
            exe_path = root.join(WINDOWS_EXE_PATH);
            data_path = root.join(WINDOWS_DATA_PATH);
        } else if cfg!(all(target_os="macos", target_arch="x86_64")) {
            exe_path = root.join(MACOS_EXE_PATH);
            data_path = root.join(MACOS_DATA_PATH);
        } else if cfg!(all(target_os="linux", target_arch="x86_64")) {
            exe_path = root.join(LINUX_EXE_PATH);
            data_path = root.join(LINUX_DATA_PATH);
        } else {
            panic!("Invalid target os or architecture");
        }
        Game {
            exe_path,
            managed_path: data_path.join(MANAGED_PATH),
            assembly_path: data_path.join(ASSEMBLY_PATH),
            assembly_backup_path: data_path.join(ASSEMBLY_BACKUP_PATH),
            mods_path: data_path.join(MODS_PATH),
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
            .arg("--dependency-missing-throw=0")
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
        let pdb_path = dir.path().join("MONOMODDED_Assembly-CSharp.pdb");
        let mdb_path = dir.path().join("MONOMODDED_Assembly-CSharp.mdb");
        if pdb_path.exists() {
            fs::remove_file(pdb_path).unwrap();
        }
        if mdb_path.exists() {
            fs::remove_file(mdb_path).unwrap();
        }
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
                Err(e) => return Err(format!("Failed to copy {}: {}", path.display(), e)),
            }
        }

        dir.close().unwrap();
        Ok(())
    }

    pub fn get_mod(&self, mod_id: &str) -> Option<Mod> {
        let mod_path = self.mods_path.join(mod_id);
        if mod_path.exists() {
            Some(Mod::new(mod_path.as_path()))
        } else {
            None
        }
    }

    pub fn get_mods_dir(&self) -> &Path {
        self.mods_path.as_path()
    }

    pub fn make_mods_dir(&self) -> io::Result<()> {
        fs::create_dir_all(&self.mods_path)
    }
}

pub struct Mod {
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: Version,
    pub description: String,
    pub description_long: String,
    pub dependencies: Vec<ModDependency>,
}

impl Mod {
    pub fn new(mod_dir: &Path) -> Mod {
        // Load and validate the mod's manifest
        let manifest: serde_json::Value =
            serde_json::from_reader(File::open(mod_dir.join("manifest.json")).unwrap()).unwrap();
        Mod {
            id: mod_dir.file_name().unwrap().to_str().unwrap().to_string(),
            name: manifest["name"].to_string(),
            author: manifest["author"].to_string(),
            version: Version::parse(manifest["version"].as_str().unwrap()).unwrap(),
            description: manifest["description"].to_string(),
            description_long: manifest["description_long"].to_string(),
            dependencies: manifest["dependencies"]
                .as_array()
                .unwrap()
                .iter()
                .map(|dependency| ModDependency::parse(dependency.as_str().unwrap()).unwrap())
                .collect(),
        }
    }
}

pub struct ModDependency {
    pub id: String,
    pub operator: Option<ModDependencyOperator>,
    pub version: Option<Version>,
}

impl ModDependency {
    pub fn new(
        id: String,
        operator: Option<ModDependencyOperator>,
        version: Option<Version>,
    ) -> ModDependency {
        ModDependency {
            id,
            operator,
            version,
        }
    }

    pub fn parse(dependency_string: &str) -> Result<ModDependency, String> {
        lazy_static! {
            static ref VERSION_REGEX: Regex = Regex::new(MOD_DEPENDENCY_VERSION).unwrap();
        }
        let captures = VERSION_REGEX.captures(dependency_string).unwrap();
        Ok(ModDependency::new(
            captures.get(1).unwrap().as_str().to_string(),
            match captures.get(2) {
                Some(op) => Some(match op.as_str() {
                    "<" => ModDependencyOperator::LessThan,
                    "<=" => ModDependencyOperator::LessThanOrEqual,
                    ">" => ModDependencyOperator::GreaterThan,
                    ">=" => ModDependencyOperator::GreaterThanOrEqual,
                    "==" => ModDependencyOperator::Equal,
                    _ => panic!("Unexpected dependency operator"),
                }),
                None => None,
            },
            match captures.get(3) {
                Some(version) => Some(match Version::parse(version.as_str()) {
                    Ok(v) => v,
                    Err(e) => return Err(format!("Invalid version specifier: {}", e)),
                }),
                None => None,
            },
        ))
    }
}

pub enum ModDependencyOperator {
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
}
