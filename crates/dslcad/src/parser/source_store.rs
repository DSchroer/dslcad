use crate::parser::{ParseError, Reader};
use std::cell::RefCell;
use std::collections::LinkedList;
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct DocId {
    path: String,
}

impl DocId {
    fn new(path: String) -> Self {
        Self { path }
    }

    pub fn to_path(&self) -> &Path {
        return Path::new(&self.path);
    }

    pub fn to_str(&self) -> &str {
        &self.path
    }
}

impl Display for DocId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_str())
    }
}

pub struct SourceStore {
    reader: Box<dyn Reader>,
    documents: RefCell<LinkedList<(DocId, String)>>,
}

impl SourceStore {
    pub fn new(reader: Box<dyn Reader>) -> Self {
        Self {
            reader,
            documents: RefCell::new(LinkedList::new()),
        }
    }

    pub fn read(&self, id: &DocId) -> &str {
        self.try_read(id)
            .map(|(_, v)| v)
            .expect("source for key is missing store")
    }

    pub fn forge_id(&self, id: String) -> Result<&DocId, ParseError> {
        let id = self.reader.normalize(Path::new(&id));
        let id = DocId::new(id.display().to_string());

        if let Some((id, _)) = self.try_read(&id) {
            Ok(id)
        } else {
            let data = self
                .reader
                .read(id.to_path())
                .map_err(|_| ParseError::NoSuchFile(id.to_path().to_path_buf()))?;
            self.documents.borrow_mut().push_back((id, data));

            let (key, _) = self.try_read_back().expect("item not found after insert");
            Ok(key)
        }
    }

    fn try_read(&self, id: &DocId) -> Option<&(DocId, String)> {
        let cells = unsafe {
            // SAFETY: safe since we only ever append to the end of the list
            self.documents.try_borrow_unguarded()
        }
        .unwrap();

        cells.iter().find(|(key, _)| key == id)
    }

    fn try_read_back(&self) -> Option<&(DocId, String)> {
        let cells = unsafe {
            // SAFETY: safe since we only ever append to the end of the list
            self.documents.try_borrow_unguarded()
        }
        .unwrap();

        cells.back()
    }
}
