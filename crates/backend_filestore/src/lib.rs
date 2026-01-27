use vfs::{MemoryFS, PhysicalFS, VfsError, VfsPath, VfsResult};
use gravitydb::KVStore;
use std::path::Path;
use thiserror::Error;
pub mod cli_helpers;

pub struct FsKvStore {
  base_path: VfsPath,
}

impl KVStore<FileStoreError> for FsKvStore
{
  fn create_bucket(&mut self, key: &[u8]) -> Result<(), FileStoreError> {
    Ok(self.key_to_path(key)?.create_dir_all()?)
  }

  fn delete_record(&mut self, key: &[u8]) -> Result<(), FileStoreError> {
    Ok(self.key_to_path(key)?.remove_file()?)
  }

  fn store_record(&mut self, key: &[u8], value: &[u8]) -> Result<(), FileStoreError> {
    Ok(self.key_to_path(key)?.create_file()?.write_all(value)?)
  }

  fn fetch_record(&self, key: &[u8]) -> Result<Vec<u8>, FileStoreError> {
    let mut content = vec![];
    self.key_to_path(key)?.open_file()?.read_to_end(&mut content)?;
    Ok(content)
  }

  fn list_records(&self, from: &[u8], to: &[u8]) -> Result<Vec<Vec<u8>>, FileStoreError> {
    let to_path = if to.len() != 0 {
      self.key_to_path(to)?
    } else {
      let mut to: Vec<u8> = from.to_vec();
      *to.last_mut().unwrap() += 1;
      self.key_to_path(&to)?
    };
    let from_path = self.key_to_path(from)?;
    let base = match longest_shared_path(&from_path, &to_path) {
      Some(base) => base,
      None => return Err(FileStoreError::InvalidParameters),
    };
    Ok(list_files(&base)?
      .into_iter()
      .filter(|key| **key < *from || **key > *to)
      .collect())
  }

  fn exists(&self, key: &[u8]) -> Result<bool, FileStoreError> {
    Ok(self.key_to_path(key)?.exists()?)
  }
}

impl FsKvStore {
  fn key_to_path(&self, key: &[u8]) -> Result<VfsPath, FileStoreError> {
    let mut path = self.base_path.clone();
    for component in String::from_utf8_lossy(key).split("/") {
      path = path.join(component)?;
    }
    Ok(path)
  }

  pub fn open(path: &Path) -> Result<Self, FileStoreError> {
    let root = VfsPath::new(PhysicalFS::new(path.to_path_buf()));

    if !root.is_dir()? {
      return Err(FileStoreError::MalformedDB);
    }

    let check_dirs = ["nodes", "edges", "props", "indexes"];
    for dir in &check_dirs {
      if !root.join(dir)?.is_dir()? {
        return Err(FileStoreError::MalformedDB);
      }
    }

    Ok(FsKvStore {
      base_path: root,
    })
  }

  pub fn init(path: &Path) -> Result<Self, FileStoreError> {
    let root = VfsPath::new(PhysicalFS::new(path.to_path_buf()));
    if !root.is_dir()? {
      if root.exists()? {
        return Err(FileStoreError::MalformedDB);
      } else {
        root.create_dir_all()?;
      }
    }

    let check_dirs = ["nodes", "edges", "props", "indexes"];
    for dir in &check_dirs {
      root.join(dir)?.create_dir_all()?;
    }

    Ok(FsKvStore {
      base_path: root,
    })
  }

  pub fn from_memory() -> Result<Self, FileStoreError> {
    let root = VfsPath::new(MemoryFS::new());

    let check_dirs = ["nodes", "edges", "props", "indexes"];
    for dir in &check_dirs {
      root.join(dir)?.create_dir_all()?;
    }

    Ok(FsKvStore { base_path: root })
  }

  pub fn get_root(self) -> VfsPath {
    self.base_path
  }
}

#[derive(Error, Debug)]
pub enum FileStoreError {
  #[error("wrongly formatted database at path TODO")]
  MalformedDB,
  #[error("io error")]
  Io { #[from] source: std::io::Error },
  #[error("vfs error")]
  Vfs { #[from] source: VfsError },
  #[error("invalid input parameters")]
  InvalidParameters,
}

fn list_files(dir: &VfsPath) -> VfsResult<Vec<Vec<u8>>> {
  let mut result = vec![];

  if dir.is_dir()? {
    for path in dir.read_dir()? {
      if path.is_dir()? {
        result.append(&mut list_files(&path)?);
      } else {
        let path = path.as_str();
        let path = match path.strip_prefix("/") {
          Some(path) => path,
          None => path,
        };
        result.push(path.as_bytes().to_vec());
      }
    }
  }
  Ok(result)
}

fn longest_shared_path(path1: &VfsPath, path2: &VfsPath) -> Option<VfsPath> {
  let s1 = path1.as_str();
  let s2 = path2.as_str();

  let mut shared = String::new();

  for (c1, c2) in s1.chars().zip(s2.chars()) {
    if c1 == c2 {
      shared.push(c1);
    } else {
      break;
    }
  }

  if !shared.is_empty() {
    let shared = path1.root().join(shared).ok()?;
    if shared.is_dir().ok()? {
      Some(shared)
    } else {
      Some(shared.parent())
    }
  } else {
    None
  }
}
