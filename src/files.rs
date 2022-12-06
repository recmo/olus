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
    pub fn new() -> Self {
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

impl Index<FileId> for Files {
    type Output = File;

    fn index(&self, id: FileId) -> &Self::Output {
        &self.files[id.0]
    }
}

impl ariadne::Cache<FileId> for &Files {
    fn fetch(&mut self, id: &FileId) -> Result<&Source, Box<dyn std::fmt::Debug + '_>> {
        Ok(&self[*id].source)
    }

    fn display<'a>(&self, id: &'a FileId) -> Option<Box<dyn std::fmt::Display + 'a>> {
        let path = self[*id].name().display().to_string();
        Some(Box::new(path))
    }
}

impl FileId {
    pub fn span(&self, range: Range<usize>) -> Span {
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
        let source = Source::from(&contents);
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
    pub fn file(&self) -> FileId {
        self.file
    }

    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }

    pub fn label(&self) -> Label<Self> {
        Label::new(*self)
    }

    pub fn report(&self, kind: ReportKind) -> ReportBuilder<Span> {
        Report::build(kind, self.file, self.range().start)
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
