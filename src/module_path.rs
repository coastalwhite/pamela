use std::path::PathBuf;
use std::str::FromStr;
use std::fmt::Display;

#[derive(Debug)]
pub enum ModulePathType {
    Absolute,
    Relative,
}

#[derive(Debug)]
pub struct ModulePath {
    path_type: ModulePathType,
    path: PathBuf,
}

impl FromStr for ModulePath {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(());
        }

        let path_type = match s.starts_with('/') {
            true => ModulePathType::Absolute,
            false => ModulePathType::Relative,
        };

        let path = PathBuf::from(s);

        Ok(ModulePath { path_type, path })
    }
}

impl Display for ModulePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.display())
    }
}
