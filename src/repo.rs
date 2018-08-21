use fs_extra::dir;
use fs_extra::dir::CopyOptions;
use reqwest;
use reqwest::{Response, StatusCode};
use semver::Version;
use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;
use tempdir::TempDir;
use zip::ZipArchive;

use game::{Game, Mod, ModDependency, ModDependencyOperator};

const DEFAULT_MOD_REPOSITORY_URL: &'static str = "http://mods.thefansus.com/downloads";

#[derive(Deserialize)]
struct RepoModVersions {
    versions: Vec<String>,
}

pub struct Repo {
    temp_dir: TempDir,
    url: String,
}

impl Repo {
    pub fn new() -> Result<Repo, String> {
        let temp_dir;
        match TempDir::new("frangiclave-mod-repository") {
            Ok(dir) => temp_dir = dir,
            Err(e) => {
                return Err(format!(
                    "Failed to initialize temporary directory for repo: {}",
                    e
                ))
            }
        }
        Ok(Repo {
            temp_dir,
            url: DEFAULT_MOD_REPOSITORY_URL.to_string(),
        })
    }

    pub fn install_mod(&self, game: &Game, dependency: &ModDependency) -> Result<(), String> {
        // Check if the mod is already installed, and, if necessary, determine which version to
        // download
        // If no requirement is specified, get the latest version
        match game.get_mod(&dependency.id) {
            Some(installed_mod) => {
                // Mod is installed, check if the version is valid
                match &dependency.version {
                    Some(version) => {
                        let valid = match &dependency.operator {
                            Some(operator) => match operator {
                                ModDependencyOperator::LessThan => &installed_mod.version < version,
                                ModDependencyOperator::LessThanOrEqual => {
                                    &installed_mod.version <= version
                                }
                                ModDependencyOperator::GreaterThan => {
                                    &installed_mod.version > version
                                }
                                ModDependencyOperator::GreaterThanOrEqual => {
                                    &installed_mod.version >= version
                                }
                                ModDependencyOperator::Equal => &installed_mod.version == version,
                            },
                            None => panic!("Invalid dependency operator"),
                        };
                        if valid {
                            return Ok(());
                        } else {
                            return Err(format!(
                                "Invalid installed version for '{}': {}",
                                installed_mod.id, installed_mod.version
                            ));
                        }
                    }
                    None => return Ok(()), // No requirement, all is well
                }
            }
            None => (), // Mod is not installed, download it
        }

        // Download a list of available versions
        let versions_url = format!("{0}/{1}/{2}", self.url, dependency.id, "versions.json");
        let available_versions_str: Vec<String> = match get_url(&versions_url) {
            Ok(mut response) => {
                let versions: RepoModVersions = response.json().unwrap();
                versions.versions
            }
            Err(e) => return Err(format!("Request to repository failed: {}", e)),
        };
        let mut available_versions: Vec<Version> = available_versions_str
            .iter()
            .map(|v| Version::parse(v).unwrap())
            .collect();
        available_versions.sort_unstable();
        let version: String = match &dependency.operator {
            Some(op) => {
                let dependency_version = dependency.version.clone().unwrap();
                let chosen_version = match op {
                    ModDependencyOperator::LessThan => {
                        get_chosen_version(&available_versions, dependency_version, |v1, v2| {
                            v1 < v2
                        })
                    }
                    ModDependencyOperator::LessThanOrEqual => {
                        get_chosen_version(&available_versions, dependency_version, |v1, v2| {
                            v1 <= v2
                        })
                    }
                    ModDependencyOperator::GreaterThan => {
                        get_chosen_version(&available_versions, dependency_version, |v1, v2| {
                            v1 > v2
                        })
                    }
                    ModDependencyOperator::GreaterThanOrEqual => {
                        get_chosen_version(&available_versions, dependency_version, |v1, v2| {
                            v1 >= v2
                        })
                    }
                    ModDependencyOperator::Equal => {
                        get_chosen_version(&available_versions, dependency_version, |v1, v2| {
                            v1 == v2
                        })
                    }
                };
                match chosen_version {
                    Some(version) => version.to_string(),
                    None => {
                        return Err(format!(
                            "No valid version found of mod '{}'",
                            &dependency.id
                        ))
                    }
                }
            }
            None => available_versions.last().unwrap().to_string(),
        };

        // Download the requested mod's ZIP file
        let mod_zip = format!("{0}-{1}.zip", dependency.id, version);
        let mod_url = format!("{0}/{1}/{2}", self.url, dependency.id, &mod_zip);
        let mod_zip_file = match get_url_to_file(&mod_url, &self.temp_dir.path().join(&mod_zip)) {
            Ok(f) => f,
            Err(e) => return Err(format!("Request to repository failed: {}", e)),
        };
        let downloaded_mod = unzip_mod(&mod_zip_file, self.temp_dir.path(), &dependency.id);

        // Move the downloaded mod to the game directory, removing any old versions first
        let source_dir = self.temp_dir.path().join(&dependency.id);
        let destination_dir = game.get_mods_dir().join(&dependency.id);
        if destination_dir.exists() {
            match fs::remove_dir_all(&destination_dir) {
                Ok(_) => (),
                Err(e) => {
                    return Err(format!(
                        "Failed to delete eventual old versions of mod '{}': {}",
                        dependency.id, e
                    ))
                }
            };
        }
        let mut copy_options = CopyOptions::new();
        copy_options.copy_inside = true;
        match dir::copy(&source_dir, &destination_dir, &copy_options) {
            Ok(_) => (),
            Err(e) => {
                return Err(format!(
                    "Failed to copy files for mod '{}': {}",
                    dependency.id, e
                ))
            }
        }

        // Download and install every dependency
        for mod_dependency in downloaded_mod.dependencies {
            self.install_mod(game, &mod_dependency)?;
        }

        Ok(())
    }
}

fn get_chosen_version<F>(
    available_versions: &Vec<Version>,
    dependency_version: Version,
    comparator: F,
) -> Option<Version>
where
    F: Fn(&Version, &Version) -> bool,
{
    let mut candidate_version: Option<Version> = None;
    for version in available_versions {
        if !comparator(version, &dependency_version) {
            continue;
        }
        candidate_version = candidate_version
            .map(|v| {
                if comparator(&v, version) {
                    version.clone()
                } else {
                    v
                }
            })
            .or_else(|| Some(version.clone()));
    }
    candidate_version
}

fn unzip_mod(file: &File, output_dir: &Path, mod_id: &str) -> Mod {
    let mut archive = ZipArchive::new(file).unwrap();

    // Copy every file and directory from the archive
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let output_path = output_dir.join(file.sanitized_name());

        if (&*file.name()).ends_with('/') {
            fs::create_dir_all(&output_path).unwrap();
        } else {
            if let Some(p) = output_path.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p).unwrap();
                }
            }
            let mut output_file = fs::File::create(&output_path).unwrap();
            io::copy(&mut file, &mut output_file).unwrap();
        }
    }

    // Load the mod from the downloaded files
    Mod::new(&output_dir.join(mod_id))
}

fn get_url_to_file(url: &str, output_path: &Path) -> Result<File, String> {
    let mut response = get_url(url)?;

    // Download the ZIP file to the output path
    match response.status() {
        StatusCode::Ok => (),
        _ => return Err(format!("Failed to fetch '{}'", url)),
    }
    let mut output_file = match File::create(output_path) {
        Ok(mut f) => f,
        Err(e) => {
            return Err(format!(
                "Failed to create file '{}': {}",
                output_path.display(),
                e
            ))
        }
    };
    match io::copy(&mut response, &mut output_file) {
        Ok(_) => Ok(File::open(output_path).unwrap()),
        Err(e) => Err(format!(
            "Failed to download response body for '{}': {}",
            url, e
        )),
    }
}

fn get_url(url: &str) -> Result<Response, String> {
    // Build our own client to work around a bug with GZIP in the library
    // See: https://github.com/seanmonstar/reqwest/issues/328
    let request = reqwest::ClientBuilder::new()
        .gzip(false)
        .build()
        .unwrap()
        .get(url)
        .send();

    // Download the ZIP file to the output path
    match request {
        Ok(response) => match response.status() {
            StatusCode::Ok => Ok(response),
            _ => Err(format!("Failed to fetch '{}'", url)),
        },
        Err(e) => Err(format!("Failed to fetch '{}': {}", url, e)),
    }
}
