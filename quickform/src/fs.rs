//! In-memory filesystem implementation for template management
//!
//! This module provides a virtual filesystem that can:
//! - Load templates from disk into memory
//! - Manage an in-memory hierarchy of files and directories
//! - Write generated content back to disk
//!
//! # Architecture
//!
//! The filesystem is structured as a tree where:
//! - `MemFS` is the root container
//! - `DirectoryNode` represents directories containing other nodes
//! - `FileNode` represents files containing raw bytes
//!
//! # Usage
//!
//! ```no_run
//! use quickform::fs::MemFS;
//!
//! // Create a new filesystem
//! let mut fs = MemFS::new();
//!
//! // Create directories and files
//! fs.create_dir("templates").unwrap();
//! fs.write_file("templates/hello.txt", b"Hello, World!".to_vec()).unwrap();
//!
//! // Read file contents
//! let content = fs.read_file("templates/hello.txt").unwrap();
//!
//! // Write entire filesystem to disk
//! fs.write_to_disk("output/").unwrap();
//! ```
//!
//! # Error Handling
//!
//! Operations that can fail return a `Result<T, FSError>` where `FSError`
//! encompasses various failure modes like:
//! - Invalid paths
//! - Missing files or directories
//! - Permission issues
//! - I/O errors
//!
//! # Implementation Details
//!
//! The filesystem maintains creation and modification timestamps for all nodes,
//! supports nested directory structures, and handles both binary and text files.
//! All paths use forward slashes (`/`) as separators regardless of the host OS.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Error types specific to filesystem operations
#[derive(Error, Debug)]
pub enum FSError {
    #[error("Invalid path")]
    InvalidPath,
    #[error("{0} is not a directory")]
    NotADirectory(String),
    #[error("{0} already exists")]
    AlreadyExists(String),
    #[error("{0} not found")]
    NotFound(String),
    #[error("System time error: {0}")]
    SystemTimeError(#[from] std::time::SystemTimeError),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}

/// An in-memory representation of a file or directory node
#[derive(Debug, Clone)]
enum FSNode {
    File(FileNode),
    Directory(DirectoryNode),
}

/// Represents a file in the in-memory filesystem
#[derive(Debug, Clone)]
pub(crate) struct FileNode {
    /// Raw content of the file
    content: Vec<u8>,
    /// Unix timestamp of when the file was created
    #[allow(unused)]
    created: u64,
    /// Unix timestamp of when the file was last modified
    #[allow(unused)]
    modified: u64,
}

/// Represents a directory in the in-memory filesystem
#[derive(Debug, Clone)]
struct DirectoryNode {
    /// Map of child node names to their contents
    children: HashMap<String, FSNode>,
    /// Unix timestamp of when the directory was created
    #[allow(unused)]
    created: u64,
}

/// An in-memory filesystem that can be read from and written to disk
/// 
/// This struct provides a virtual filesystem that can be used to manage
/// templates and generated files in memory before writing them to disk.
#[derive(Debug, Clone)]
pub(crate) struct MemFS {
    root: DirectoryNode,
}

impl MemFS {
    /// Creates a new empty filesystem
    pub(crate) fn new() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            root: DirectoryNode {
                children: HashMap::new(),
                created: timestamp,
            },
        }
    }

    /// Reads an entire directory structure from disk into memory
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the directory to read
    ///
    /// # Returns
    ///
    /// A new MemFS instance containing the directory structure
    pub(crate) fn read_from_disk<P: AsRef<Path>>(path: P) -> Result<Self, FSError> {
        let mut fs = MemFS::new();
        fs.read_directory_recursive("", path)?;
        Ok(fs)
    }

    /// Writes a file to the specified path in the filesystem
    ///
    /// Creates parent directories as needed. If the file already exists,
    /// it will be overwritten.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the file should be written
    /// * `content` - Raw content to write to the file
    pub(crate) fn write_file(&mut self, path: &str, content: Vec<u8>) -> Result<(), FSError> {
        let components: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if components.is_empty() {
            return Err(FSError::InvalidPath);
        }

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let mut current = &mut self.root;

        // Navigate to parent directory
        for &component in components.iter().take(components.len() - 1) {
            if !current.children.contains_key(component) {
                current.children.insert(
                    component.to_string(),
                    FSNode::Directory(DirectoryNode {
                        children: HashMap::new(),
                        created: timestamp,
                    }),
                );
            }

            match current.children.get_mut(component) {
                Some(FSNode::Directory(dir)) => current = dir,
                Some(_) => return Err(FSError::NotADirectory(component.to_string())),
                None => unreachable!("We just inserted the directory"),
            }
        }

        // Insert or update the file
        let name = components.last().unwrap();
        let file_node = FSNode::File(FileNode {
            content,
            created: match current.children.get(*name) {
                Some(FSNode::File(existing)) => existing.created,
                _ => timestamp,
            },
            modified: timestamp,
        });
        
        current.children.insert(name.to_string(), file_node);
        Ok(())
    }

    /// Creates a new directory at the specified path
    ///
    /// Creates parent directories as needed. Returns an error if the path
    /// already exists.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the directory should be created
    pub(crate) fn create_dir(&mut self, path: &str) -> Result<(), FSError> {
        let components: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if components.is_empty() {
            return Err(FSError::InvalidPath);
        }

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        self.create_node(
            &components,
            FSNode::Directory(DirectoryNode {
                children: HashMap::new(),
                created: timestamp,
            }),
        )
    }

    /// Creates a new node (file or directory) at the specified path
    ///
    /// # Arguments
    ///
    /// * `components` - Path components leading to the node location
    /// * `node` - The node to create
    fn create_node(&mut self, components: &[&str], node: FSNode) -> Result<(), FSError> {
        let mut current = &mut self.root;

        // Navigate to parent directory
        for &component in components.iter().take(components.len() - 1) {
            if !current.children.contains_key(component) {
                let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

                current.children.insert(
                    component.to_string(),
                    FSNode::Directory(DirectoryNode {
                        children: HashMap::new(),
                        created: timestamp,
                    }),
                );
            }

            // Get the next directory after we're done modifying the current one
            match current.children.get_mut(component) {
                Some(FSNode::Directory(dir)) => current = dir,
                Some(_) => return Err(FSError::NotADirectory(component.to_string())),
                None => unreachable!("We just inserted the directory"),
            }
        }

        // Insert the new node
        let name = components.last().unwrap();
        if current.children.contains_key(*name) {
            return Err(FSError::AlreadyExists(name.to_string()));
        }
        current.children.insert(name.to_string(), node);
        Ok(())
    }

    /// Reads the contents of a file at the specified path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to read
    ///
    /// # Returns
    ///
    /// The raw contents of the file
    pub(crate) fn read_file(&self, path: &str) -> Result<&Vec<u8>, FSError> {
        let components: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if components.is_empty() {
            return Err(FSError::InvalidPath);
        }

        let mut current = &self.root;
        for (i, &component) in components.iter().enumerate() {
            match current.children.get(component) {
                Some(FSNode::File(file)) if i == components.len() - 1 => return Ok(&file.content),
                Some(FSNode::Directory(dir)) if i < components.len() - 1 => current = dir,
                Some(_) => return Err(FSError::NotFound(format!("Invalid path: {}", path))),
                None => return Err(FSError::NotFound(format!("{} not found", component))),
            }
        }
        Err(FSError::NotFound(format!("Path not found: {}", path)))
    }

    /// Lists the contents of a directory
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the directory to list
    ///
    /// # Returns
    ///
    /// A vector of names of the directory's contents
    #[allow(unused)]
    pub(crate) fn list_dir(&self, path: &str) -> Result<Vec<String>, FSError> {
        let components: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        let mut current = &self.root;
        for component in components {
            match current.children.get(component) {
                Some(FSNode::Directory(dir)) => current = dir,
                Some(_) => return Err(FSError::NotADirectory(component.to_string())),
                None => {
                    return Err(FSError::NotFound(format!(
                        "Directory {} not found",
                        component
                    )))
                }
            }
        }

        Ok(current.children.keys().cloned().collect())
    }

    /// Recursively reads a directory from disk into memory
    ///
    /// # Arguments
    ///
    /// * `prefix` - Virtual path prefix for the current directory
    /// * `path` - Physical path to read from
    fn read_directory_recursive<P: AsRef<Path>>(
        &mut self,
        prefix: &str,
        path: P,
    ) -> Result<(), FSError> {
        let path = path.as_ref();
        for entry in fs::read_dir(path).map_err(|e| FSError::NotFound(e.to_string()))? {
            let entry = entry.map_err(|e| FSError::NotFound(e.to_string()))?;
            let file_type = entry
                .file_type()
                .map_err(|e| FSError::NotFound(e.to_string()))?;
            let name = entry.file_name().to_string_lossy().into_owned();

            let virtual_path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", prefix, name)
            };

            if file_type.is_dir() {
                self.create_dir(&virtual_path)?;
                self.read_directory_recursive(&virtual_path, entry.path())?;
            } else if file_type.is_file() {
                let content =
                    fs::read(entry.path()).map_err(|e| FSError::NotFound(e.to_string()))?;
                self.write_file(&virtual_path, content)?;
            }
        }
        Ok(())
    }

    /// Writes the entire filesystem structure to disk
    ///
    /// # Arguments
    ///
    /// * `path` - Base path where the filesystem should be written
    pub(crate) fn write_to_disk<P: AsRef<Path>>(&self, path: P) -> Result<(), FSError> {
        let base_path = path.as_ref();
        
        // Create the root directory if it doesn't exist
        if !base_path.exists() {
            fs::create_dir_all(base_path).map_err(FSError::IOError)?;
        }

        self.write_node_to_disk("", base_path, &self.root)
    }

    /// Recursively writes a directory node and its contents to disk
    ///
    /// # Arguments
    ///
    /// * `prefix` - Virtual path prefix for the current node
    /// * `base_path` - Physical base path where contents should be written
    /// * `node` - The directory node to write
    fn write_node_to_disk(
        &self,
        prefix: &str,
        base_path: &Path,
        node: &DirectoryNode,
    ) -> Result<(), FSError> {
        for (name, child) in &node.children {
            let child_path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", prefix, name)
            };
            
            let full_path = base_path.join(name);

            match child {
                FSNode::File(file) => {
                    fs::write(&full_path, &file.content).map_err(FSError::IOError)?;
                }
                FSNode::Directory(dir) => {
                    fs::create_dir_all(&full_path).map_err(FSError::IOError)?;
                    self.write_node_to_disk(&child_path, &full_path, dir)?;
                }
            }
        }
        Ok(())
    }
}

impl Default for MemFS {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_filesystem() -> Result<(), FSError> {
        let mut fs = MemFS::new();

        // Test directory creation
        fs.create_dir("test_dir")?;
        assert!(fs.list_dir("test_dir")?.is_empty());

        // Test file creation
        fs.write_file("test_dir/hello.txt", b"Hello, World!".to_vec())?;

        // Test file reading
        assert_eq!(
            fs.read_file("test_dir/hello.txt")?,
            &b"Hello, World!".to_vec()
        );

        // Test directory listing
        assert_eq!(fs.list_dir("test_dir")?, vec!["hello.txt"]);

        Ok(())
    }

    #[test]
    fn test_read_from_disk() -> Result<(), FSError> {
        // Create a temporary directory for testing
        let temp_dir = tempdir::TempDir::new("fs_test").unwrap();
        let base_path = temp_dir.path();

        // Create a test directory structure
        let test_dir = base_path.join("test_dir");
        fs::create_dir(&test_dir).unwrap();
        fs::write(test_dir.join("file1.txt"), "Hello").unwrap();

        let nested_dir = test_dir.join("nested");
        fs::create_dir(&nested_dir).unwrap();
        fs::write(nested_dir.join("file2.txt"), "World").unwrap();

        // Read the directory into our virtual filesystem
        let fs = MemFS::read_from_disk(&base_path)?;

        // Verify the structure (order doesn't matter)
        let mut root_contents = fs.list_dir("")?;
        root_contents.sort();
        assert_eq!(root_contents, vec!["test_dir"]);

        let mut dir_contents = fs.list_dir("test_dir")?;
        dir_contents.sort();
        assert_eq!(dir_contents, vec!["file1.txt", "nested"]);

        let mut nested_contents = fs.list_dir("test_dir/nested")?;
        nested_contents.sort();
        assert_eq!(nested_contents, vec!["file2.txt"]);

        // Verify file contents
        assert_eq!(fs.read_file("test_dir/file1.txt")?, b"Hello");
        assert_eq!(fs.read_file("test_dir/nested/file2.txt")?, b"World");

        Ok(())
    }

    #[test]
    fn test_write_to_disk() -> Result<(), FSError> {
        // Create a temporary directory for testing
        let temp_dir = tempdir::TempDir::new("fs_test").unwrap();
        let base_path = temp_dir.path();

        // Create a test filesystem in memory
        let mut fs = MemFS::new();
        fs.create_dir("test_dir")?;
        fs.write_file("test_dir/file1.txt", b"Hello".to_vec())?;
        fs.create_dir("test_dir/nested")?;
        fs.write_file("test_dir/nested/file2.txt", b"World".to_vec())?;

        // Write the filesystem to disk
        fs.write_to_disk(base_path)?;

        // Verify the structure on disk
        assert!(base_path.join("test_dir").is_dir());
        assert!(base_path.join("test_dir/file1.txt").is_file());
        assert!(base_path.join("test_dir/nested").is_dir());
        assert!(base_path.join("test_dir/nested/file2.txt").is_file());

        // Verify file contents
        assert_eq!(
            fs::read(base_path.join("test_dir/file1.txt")).unwrap(),
            b"Hello"
        );
        assert_eq!(
            fs::read(base_path.join("test_dir/nested/file2.txt")).unwrap(),
            b"World"
        );

        // Test round-trip: read the written filesystem back into memory
        let fs2 = MemFS::read_from_disk(base_path)?;
        assert_eq!(fs2.read_file("test_dir/file1.txt")?, b"Hello");
        assert_eq!(fs2.read_file("test_dir/nested/file2.txt")?, b"World");

        Ok(())
    }
}
