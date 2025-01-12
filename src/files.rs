use ariadne::{Label, Report, ReportBuilder, ReportKind, Source};
use std::{
    fs::read_to_string,
    io,
    ops::{Index, Range},
    path::{Path, PathBuf},
};

pub struct Files {
    files: Vec<File>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FileId(usize);

pub struct File {
    path:     PathBuf,
    contents: String,
    source:   Source,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Span {
    file:  FileId,
    start: usize,
    end:   usize,
}

impl Files {
    #[must_use]
    pub const fn new() -> Self {
        Self { files: Vec::new() }
    }

    pub fn insert(&mut self, path: PathBuf) -> io::Result<FileId> {
        // TODO: Canonicalize path
        // TODO: Deduplicated
        let id = self.files.len();
        self.files.push(File::new(path)?);
        Ok(FileId(id))
    }
}

impl Default for Files {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<FileId> for Files {
    type Output = File;

    fn index(&self, id: FileId) -> &Self::Output {
        &self.files[id.0]
    }
}

impl ariadne::Cache<FileId> for &Files {
    type Storage = String;

    fn fetch(&mut self, id: &FileId) -> Result<&Source, Box<dyn std::fmt::Debug + '_>> {
        Ok(&self[*id].source)
    }

    fn display<'a>(&self, id: &'a FileId) -> Option<Box<dyn std::fmt::Display + 'a>> {
        let path = self[*id].name().display().to_string();
        Some(Box::new(path))
    }
}

impl FileId {
    #[must_use]
    pub const fn span(&self, range: Range<usize>) -> Span {
        Span {
            file:  *self,
            start: range.start,
            end:   range.end,
        }
    }
}

impl File {
    fn new(path: PathBuf) -> io::Result<Self> {
        let contents = read_to_string(&path)?;
        let source = Source::from(contents.clone());
        Ok(Self {
            path,
            contents,
            source,
        })
    }

    pub fn name(&self) -> &Path {
        &self.path
    }

    pub fn contents(&self) -> &str {
        &self.contents
    }
}

impl Span {
    #[must_use]
    pub const fn file(&self) -> FileId {
        self.file
    }

    #[must_use]
    pub const fn range(&self) -> Range<usize> {
        self.start..self.end
    }

    #[must_use]
    pub fn label(&self) -> Label<Self> {
        Label::new(*self)
    }

    #[must_use]
    pub fn report<'a>(&self, kind: ReportKind<'a>) -> ReportBuilder<'a, Self> {
        Report::build(kind, *self)
    }
}

impl ariadne::Span for Span {
    type SourceId = FileId;

    fn source(&self) -> &Self::SourceId {
        &self.file
    }

    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }
}
