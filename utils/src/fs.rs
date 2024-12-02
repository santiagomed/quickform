use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

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

// Represents either a file or directory
#[derive(Debug)]
enum FSNode {
    File(FileNode),
    Directory(DirectoryNode),
}

// Represents a file and its contents
#[derive(Debug)]
struct FileNode {
    content: Vec<u8>,
    #[allow(unused)]
    created: u64,
    #[allow(unused)]
    modified: u64,
}

// Represents a directory and its children
#[derive(Debug)]
struct DirectoryNode {
    children: HashMap<String, FSNode>,
    #[allow(unused)]
    created: u64,
}

// Main filesystem structure
#[derive(Debug)]
pub struct MemFS {
    root: DirectoryNode,
}

impl MemFS {
    pub fn new() -> Self {
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

    /// Reads a real directory from the filesystem into memory
    pub fn read_from_disk<P: AsRef<Path>>(path: P) -> Result<Self, FSError> {
        let mut fs = MemFS::new();
        fs.read_directory_recursive("", path)?;
        Ok(fs)
    }

    // Create a new file at the specified path
    pub fn create_file(&mut self, path: &str, content: Vec<u8>) -> Result<(), FSError> {
        let components: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if components.is_empty() {
            return Err(FSError::InvalidPath);
        }

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        self.create_node(
            &components,
            FSNode::File(FileNode {
                content,
                created: timestamp,
                modified: timestamp,
            }),
        )
    }

    // Create a new directory at the specified path
    pub fn create_dir(&mut self, path: &str) -> Result<(), FSError> {
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

    // Read a file's contents
    pub fn read_file(&self, path: &str) -> Result<&Vec<u8>, FSError> {
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

    // List contents of a directory
    pub fn list_dir(&self, path: &str) -> Result<Vec<String>, FSError> {
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

    /// Reads a real directory from the filesystem into memory
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
                self.create_file(&virtual_path, content)?;
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
        fs.create_file("test_dir/hello.txt", b"Hello, World!".to_vec())?;

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
        let fs = MemFS::read_from_disk(base_path)?;

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
}
