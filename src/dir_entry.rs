use std::{
    fs::{FileType, Metadata},
    path::{Path, PathBuf},
};

use once_cell::unsync::OnceCell;

use crate::config::Config;
use crate::filesystem::strip_current_dir;

enum DirEntryInner {
    Normal(ignore::DirEntry),
    BrokenSymlink{ path: PathBuf, depth: usize },
}

pub struct DirEntry {
    inner: DirEntryInner,
    metadata: OnceCell<Option<Metadata>>,
}

impl DirEntry {
    #[inline]
    pub fn normal(e: ignore::DirEntry) -> Self {
        Self {
            inner: DirEntryInner::Normal(e),
            metadata: OnceCell::new(),
        }
    }

    pub fn broken_symlink(path: PathBuf, depth: usize) -> Self {
        Self {
            inner: DirEntryInner::BrokenSymlink{ path, depth },
            metadata: OnceCell::new(),
        }
    }

    pub fn path(&self) -> &Path {
        match &self.inner {
            DirEntryInner::Normal(e) => e.path(),
            DirEntryInner::BrokenSymlink{ path: pathbuf, .. } => pathbuf.as_path(),
        }
    }

    pub fn into_path(self) -> PathBuf {
        match self.inner {
            DirEntryInner::Normal(e) => e.into_path(),
            DirEntryInner::BrokenSymlink{ path: p, .. } => p,
        }
    }

    /// Returns the path as it should be presented to the user.
    pub fn stripped_path(&self, config: &Config) -> &Path {
        if config.strip_cwd_prefix {
            strip_current_dir(self.path())
        } else {
            self.path()
        }
    }

    /// Returns the path as it should be presented to the user.
    pub fn into_stripped_path(self, config: &Config) -> PathBuf {
        if config.strip_cwd_prefix {
            self.stripped_path(config).to_path_buf()
        } else {
            self.into_path()
        }
    }

    pub fn file_type(&self) -> Option<FileType> {
        match &self.inner {
            DirEntryInner::Normal(e) => e.file_type(),
            DirEntryInner::BrokenSymlink{ .. } => self.metadata().map(|m| m.file_type()),
        }
    }

    pub fn metadata(&self) -> Option<&Metadata> {
        self.metadata
            .get_or_init(|| match &self.inner {
                DirEntryInner::Normal(e) => e.metadata().ok(),
                DirEntryInner::BrokenSymlink{ path, .. } => path.symlink_metadata().ok(),
            })
            .as_ref()
    }

    pub fn depth(&self) -> usize {
        match &self.inner {
            DirEntryInner::Normal(e) => e.depth(),
            DirEntryInner::BrokenSymlink{ depth, .. } => *depth,
        }
    }
}

impl PartialEq for DirEntry {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.path() == other.path()
    }
}
impl Eq for DirEntry {}

impl PartialOrd for DirEntry {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.path().partial_cmp(other.path())
    }
}

impl Ord for DirEntry {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path().cmp(other.path())
    }
}
