use std::{env, path::PathBuf};

#[derive(Debug, Clone)]
pub struct PackageResolver {
    package_directories: Vec<PathBuf>,
    current_workdir: PathBuf
}

impl PackageResolver {
    pub fn new() -> Self {
        let current_workdir = env::current_dir().unwrap();
        let mut package_directories = vec![
            current_workdir.join("amber_modules"),
            home::home_dir().unwrap().join("amber_modules"),
            "/usr/lib/amber_modules".into()
        ];
        if let Ok(amber_path) = env::var("AMBER_PATH") {
            let amber_paths = amber_path.split(':')
                .map(|path| path.to_string())
                .collect::<Vec<String>>();
            for custom_dir in amber_paths.iter() {
                package_directories.insert(0, PathBuf::from(custom_dir).join("amber_modules"));
            }
        }

        let package_directories: Vec<PathBuf> = package_directories.iter()
            .filter(|pkgdir| pkgdir.is_dir())
            .map(|pkgdir| pkgdir.clone())
            .collect();

        Self {
            package_directories,
            current_workdir
        }
    }

    pub fn resolve<I: ToString>(&self, import: I) -> Option<PathBuf> {        
        let import = import.to_string();

        fn ponder(mut import: PathBuf) -> Option<PathBuf> {
            if import.is_dir() {
                return Some(import.join("main.ab"));
            }

            import.set_extension("ab");
            if import.is_file() {
                return Some(import);
            }
            
            import.set_extension("");
            import = import.join("main.ab");
            if import.is_file() {
                return Some(import);
            }
            None
        }

        if import.starts_with('.') {
            println!("{:?}", self.current_workdir.join(&import.trim_start_matches("./")));
            if let Some(found) = ponder(self.current_workdir.join(&import.trim_start_matches("./"))) {
                return Some(found);
            }
        }

        for pkgdir in self.package_directories.iter() {
            if let Some(found) = ponder(pkgdir.join(&import)) {
                return Some(found);
            }
        }

        None
    }
}