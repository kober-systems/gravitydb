use std::fs;
use gravitydb::KVStore;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use thiserror::Error;
pub mod cli_helpers;

pub struct FsKvStore {
  base_path: PathBuf,
}

impl KVStore<FileStoreError> for FsKvStore
{
  fn create_bucket(&mut self, key: &[u8]) -> Result<(), FileStoreError> {
    Ok(std::fs::create_dir_all(self.key_to_path(key))?)
  }

  fn delete_record(&mut self, key: &[u8]) -> Result<(), FileStoreError> {
    Ok(std::fs::remove_file(self.key_to_path(key))?)
  }

  fn store_record(&mut self, key: &[u8], value: &[u8]) -> Result<(), FileStoreError> {
    Ok(std::fs::write(self.key_to_path(key), value)?)
  }

  fn fetch_record(&self, key: &[u8]) -> Result<Vec<u8>, FileStoreError> {
    Ok(std::fs::read(self.key_to_path(key))?)
  }

  fn list_records(&self, from: &[u8], to: &[u8]) -> Result<Vec<Vec<u8>>, FileStoreError> {
    let to_path = if to.len() != 0 {
      self.key_to_path(to)
    } else {
      let mut to: Vec<u8> = from.to_vec();
      *to.last_mut().unwrap() += 1;
      self.key_to_path(&to)
    };
    let from_path = self.key_to_path(from);
    let base = match longest_shared_path(&from_path, &to_path) {
      Some(base) => base,
      None => { return Err(FileStoreError::InvalidParameters); },
    };
    Ok(list_files(&base)?
      .into_iter()
      .filter(|key| **key < *from || **key > *to)
      .collect())
  }

  fn exists(&self, key: &[u8]) -> Result<bool, FileStoreError> {
    Ok(self.key_to_path(key).exists())
  }
}

impl FsKvStore {
  fn key_to_path(&self, key: &[u8]) -> PathBuf {
    let path = Path::new(OsStr::from_bytes(key));
    PathBuf::from(self.base_path.join(path))
  }

  pub fn open(path: &Path) -> Result<Self, FileStoreError> {
    if !path.is_dir() {
      return Err(FileStoreError::MalformedDB);
    }
    if !&path.join("nodes/").is_dir() ||
      !&path.join("edges/").is_dir() ||
      !&path.join("props/").is_dir() ||
      !&path.join("indexes/").is_dir() {
        return Err(FileStoreError::MalformedDB);
    }

    Ok(FsKvStore {
      base_path: path.to_path_buf(),
    })
  }

  pub fn init(path: &Path) -> Result<Self, FileStoreError> {
    if !path.is_dir() {
      if path.exists() {
        return Err(FileStoreError::MalformedDB);
      } else {
        fs::create_dir_all(&path)?;
      }
    }

    fs::create_dir_all(&path.join("nodes/"))?;
    fs::create_dir_all(&path.join("edges/"))?;
    fs::create_dir_all(&path.join("props/"))?;
    fs::create_dir_all(&path.join("indexes/"))?;

    Ok(FsKvStore {
      base_path: path.to_path_buf(),
    })
  }
}

#[derive(Error, Debug)]
pub enum FileStoreError {
  #[error("wrongly formatted database at path TODO")]
  MalformedDB,
  #[error("io error")]
  Io { #[from] source: std::io::Error },
  #[error("invalid input parameters")]
  InvalidParameters,
}

fn list_files(dir: &Path) -> std::io::Result<Vec<Vec<u8>>> {
  let mut result = vec![];

  if dir.is_dir() {
    for entry in fs::read_dir(dir)? {
      let entry = entry?;
      let path = entry.path();
      if path.is_dir() {
        result.append(&mut list_files(&path)?);
      } else {
        result.push(path.to_string_lossy().as_bytes().to_vec());
      }
    }
  }
  Ok(result)
}

fn longest_shared_path(path1: &Path, path2: &Path) -> Option<PathBuf> {
  let components1: Vec<&str> = path1.components().filter_map(|c| c.as_os_str().to_str()).collect();
  let components2: Vec<&str> = path2.components().filter_map(|c| c.as_os_str().to_str()).collect();

  let mut shared_components = Vec::new();

  for (c1, c2) in components1.iter().zip(components2.iter()) {
    if c1 == c2 {
      shared_components.push(*c1);
    } else {
      break;
    }
  }

  if !shared_components.is_empty() {
    Some(shared_components.iter().collect::<PathBuf>())
  } else {
    None
  }
}
