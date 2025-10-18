use std::error::Error;
use std::path::Path;
use std::process::Command;

pub struct PackageManager {
    pub name: String,
    pub packages: usize,
}

impl PackageManager {
    pub fn new(name: &str, packages: usize) -> Self {
        PackageManager {
            name: String::from(name),
            packages: packages,
        }
    }
}

pub struct PackageManagers(pub Vec<PackageManager>);

impl PackageManagers {
    pub fn get() -> Result<Self, Box<dyn Error>> {
        let mut to_return = Vec::new();

        let mut add = |package_manager: &str, command: &str| {
            if let Ok(output) = Command::new(package_manager).arg(command).output() {
                to_return.push(PackageManager::new(package_manager, {
                    let stdout_string = String::from_utf8(output.stdout).unwrap();
                    let stdout_lines: Vec<&str> = stdout_string.split("\n").collect();
                    stdout_lines.len() - 1
                }));
            }
        };

        add("kiss", "l");
        add("pacman", "-Qq --color never");
        add("dpkg", "-f '.\n' -W");
        add("rpm", "-qa");
        add("xbps-query", "-l");
        add("apk", "info");
        add("opkg", "list-installed");
        add("pacman-g2", "-Q");
        add("lvu", "installed");
        add("tce-status", "-i");
        add("pkg-info", "");
        add("tazpkg", "list");
        add("sorcery", "installed");
        add("alps", "showinstalled");
        add("butch", "list");
        add("mine", "-q");
        add("dnf", "list installed");
        add("apt", "list --installed");
        add("flatpak", "list");
        add("cargo", "install --list");
        add("pip", "list");
        add("snap", "list");
        add("winget", "list");

        Ok(PackageManagers(to_return))
    }
}
