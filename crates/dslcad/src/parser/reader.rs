use std::path::{Path, PathBuf};

pub trait Reader {
    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, std::io::Error>;
    fn read(&self, path: &Path) -> Result<String, std::io::Error>;

    /// Resolve `path` relative to the importing document and normalize it into
    /// a canonical path. Readers that have access to the filesystem look up
    /// paths beginning with `@` in a `modules` directory and resolve everything
    /// else relative to the importing document. Returns `None` if the path
    /// could not be found. The default implementation only resolves paths
    /// relative to the importing document.
    fn normalize(&self, importer: &Path, path: &Path) -> Option<PathBuf> {
        Some(importer.parent().unwrap_or(Path::new("")).join(path))
    }
}
