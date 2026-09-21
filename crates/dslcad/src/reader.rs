use crate::parser::Reader;
use std::fs;
use std::io::{Error, Read};
use std::path::{Path, PathBuf};

pub struct FsReader;

impl Reader for FsReader {
    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, Error> {
        fs::read(path)
    }

    fn read(&self, path: &Path) -> Result<String, Error> {
        fs::read_to_string(path)
    }

    fn normalize(&self, importer: &Path, path: &Path) -> Option<PathBuf> {
        Some(resolve(importer, path)?.canonicalize().unwrap())
    }
}

/// Resolves a path expression from the importing document. Paths beginning
/// with `@` are looked up in the nearest `modules` directory (node style),
/// anything else is relative to the importing document.
fn resolve(importer: &Path, path: &Path) -> Option<PathBuf> {
    match path.to_str().and_then(|path| path.strip_prefix('@')) {
        Some(module) => {
            let module = module.strip_prefix('/').unwrap_or(module);
            find_module(importer, Path::new(module))
        }
        None => Some(importer.parent().unwrap_or(Path::new("")).join(path)),
    }
}

/// Walks up from the importing document's directory looking for a `modules`
/// directory that contains the requested module, node style.
fn find_module(importer: &Path, module: &Path) -> Option<PathBuf> {
    let mut directory = importer.parent();
    while let Some(current) = directory {
        let candidate = current.join("modules").join(module);
        if candidate.is_file() {
            return Some(candidate);
        }
        directory = current.parent();
    }
    None
}

pub struct StdinReader {
    source: String,
}

impl StdinReader {
    pub fn new() -> Result<Self, Error> {
        let mut source = String::new();
        std::io::stdin().read_to_string(&mut source)?;
        Ok(Self { source })
    }
}

impl Reader for StdinReader {
    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, Error> {
        if path == Path::new("-") {
            Ok(self.source.as_bytes().to_vec())
        } else {
            fs::read(path)
        }
    }

    fn read(&self, path: &Path) -> Result<String, Error> {
        if path == Path::new("-") {
            Ok(self.source.clone())
        } else {
            fs::read_to_string(path)
        }
    }

    fn normalize(&self, importer: &Path, path: &Path) -> Option<PathBuf> {
        if path == Path::new("-") {
            Some(PathBuf::from("-"))
        } else {
            Some(resolve(importer, path)?.canonicalize().unwrap())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("dslcad_{}_{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn it_resolves_relative_paths_from_the_importer() {
        assert_eq!(
            resolve(
                Path::new("/project/src/part.ds"),
                Path::new("./lib/gear.ds")
            ),
            Some(PathBuf::from("/project/src/lib/gear.ds"))
        );
    }

    #[test]
    fn it_finds_modules_by_walking_up() {
        let root = scratch("modules");
        let importer = root.join("src").join("parts").join("part.ds");
        fs::create_dir_all(importer.parent().unwrap()).unwrap();

        let module = root.join("modules").join("stamp").join("gear.ds");
        fs::create_dir_all(module.parent().unwrap()).unwrap();
        fs::write(&module, "gear();").unwrap();

        assert_eq!(
            resolve(&importer, Path::new("@stamp/gear.ds")),
            Some(module.clone())
        );

        let closer = root
            .join("src")
            .join("modules")
            .join("stamp")
            .join("gear.ds");
        fs::create_dir_all(closer.parent().unwrap()).unwrap();
        fs::write(&closer, "gear();").unwrap();

        assert_eq!(
            resolve(&importer, Path::new("@stamp/gear.ds")),
            Some(closer)
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn it_returns_none_when_no_module_exists() {
        let root = scratch("no_module");
        let importer = root.join("part.ds");

        assert_eq!(resolve(&importer, Path::new("@stamp/gear.ds")), None);

        let _ = fs::remove_dir_all(&root);
    }
}
