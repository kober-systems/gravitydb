use std::fs;
use gravity::KVStore;
use std::io::Error;
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

  fn list_records(&self, key: &[u8]) -> Result<Vec<Vec<u8>>, FileStoreError> {
    let iter: Vec<Vec<u8>> = fs::read_dir(self.key_to_path(key))?.into_iter().filter_map(|entry| {
      match entry {
        Ok(entry) => Some(entry.file_name().into_encoded_bytes()),
        Err(_) => None,
      }
    }).collect();
    Ok(iter)
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
}

