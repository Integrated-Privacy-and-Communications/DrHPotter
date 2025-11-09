//! Fake in-memory filesystem

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Fake filesystem that exists only in memory
pub struct FakeFilesystem {
    files: HashMap<PathBuf, String>,
    dirs: Vec<PathBuf>,
}

impl FakeFilesystem {
    /// Create a new fake filesystem with common Linux directory structure
    pub fn new() -> Self {
        let mut fs = Self {
            files: HashMap::new(),
            dirs: Vec::new(),
        };

        // Create common directories
        fs.create_dir("/");
        fs.create_dir("/root");
        fs.create_dir("/home");
        fs.create_dir("/etc");
        fs.create_dir("/var");
        fs.create_dir("/tmp");
        fs.create_dir("/usr");
        fs.create_dir("/bin");
        fs.create_dir("/sbin");

        // Create common files with realistic content
        fs.create_file(
            "/etc/passwd",
            "root:x:0:0:root:/root:/bin/bash\n\
             daemon:x:1:1:daemon:/usr/sbin:/usr/sbin/nologin\n\
             bin:x:2:2:bin:/bin:/usr/sbin/nologin\n\
             sys:x:3:3:sys:/dev:/usr/sbin/nologin\n\
             sync:x:4:65534:sync:/bin:/bin/sync\n\
             www-data:x:33:33:www-data:/var/www:/usr/sbin/nologin\n\
             nobody:x:65534:65534:nobody:/nonexistent:/usr/sbin/nologin\n",
        );

        fs.create_file(
            "/etc/shadow",
            "root:$6$rounds=656000$YT...:19000:0:99999:7:::\n\
             daemon:*:18375:0:99999:7:::\n\
             bin:*:18375:0:99999:7:::\n",
        );

        fs.create_file(
            "/etc/hosts",
            "127.0.0.1\tlocalhost\n\
             127.0.1.1\thoneypot\n\
             \n\
             ::1     localhost ip6-localhost ip6-loopback\n\
             ff02::1 ip6-allnodes\n\
             ff02::2 ip6-allrouters\n",
        );

        fs.create_file(
            "/etc/hostname",
            "honeypot\n",
        );

        fs.create_file(
            "/etc/os-release",
            "PRETTY_NAME=\"Ubuntu 22.04.1 LTS\"\n\
             NAME=\"Ubuntu\"\n\
             VERSION_ID=\"22.04\"\n\
             VERSION=\"22.04.1 LTS (Jammy Jellyfish)\"\n\
             VERSION_CODENAME=jammy\n\
             ID=ubuntu\n\
             ID_LIKE=debian\n",
        );

        fs.create_file(
            "/root/.bashrc",
            "# .bashrc\n\
             \n\
             # If not running interactively, don't do anything\n\
             case $- in\n\
                 *i*) ;;\n\
                   *) return;;\n\
             esac\n",
        );

        fs.create_file(
            "/root/.bash_history",
            "ls -la\n\
             cd /tmp\n\
             wget http://example.com/script.sh\n\
             chmod +x script.sh\n\
             ./script.sh\n",
        );

        fs
    }

    /// Create a directory
    fn create_dir(&mut self, path: &str) {
        self.dirs.push(PathBuf::from(path));
    }

    /// Create a file with content
    fn create_file(&mut self, path: &str, content: &str) {
        self.files.insert(PathBuf::from(path), content.to_string());
    }

    /// Check if a directory exists
    pub fn dir_exists(&self, path: &Path) -> bool {
        self.dirs.iter().any(|d| d == path)
    }

    /// List directory contents
    pub fn list_dir(&self, path: &Path) -> Vec<String> {
        let path_str = path.to_string_lossy();

        // Get subdirectories
        let mut entries: Vec<String> = self
            .dirs
            .iter()
            .filter_map(|d| {
                let d_str = d.to_string_lossy();
                if let Some(parent) = d.parent() {
                    if parent.to_string_lossy() == path_str && d_str != path_str {
                        d.file_name().map(|n| n.to_string_lossy().to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        // Get files in this directory
        for (file_path, _) in &self.files {
            if let Some(parent) = file_path.parent() {
                if parent.to_string_lossy() == path_str {
                    if let Some(name) = file_path.file_name() {
                        entries.push(name.to_string_lossy().to_string());
                    }
                }
            }
        }

        entries.sort();
        entries.dedup();
        entries
    }

    /// Read a file
    pub fn read_file(&self, path: &Path) -> Option<&str> {
        self.files.get(path).map(|s| s.as_str())
    }

    /// Write a file (for honeypot purposes, we log but don't actually store)
    pub fn write_file(&mut self, path: PathBuf, content: String) {
        self.files.insert(path, content);
    }
}

impl Default for FakeFilesystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filesystem_creation() {
        let fs = FakeFilesystem::new();
        assert!(fs.dir_exists(Path::new("/root")));
        assert!(fs.dir_exists(Path::new("/etc")));
    }

    #[test]
    fn test_read_passwd() {
        let fs = FakeFilesystem::new();
        let passwd = fs.read_file(Path::new("/etc/passwd"));
        assert!(passwd.is_some());
        assert!(passwd.unwrap().contains("root"));
    }

    #[test]
    fn test_list_root() {
        let fs = FakeFilesystem::new();
        let entries = fs.list_dir(Path::new("/"));
        assert!(entries.contains(&"root".to_string()));
        assert!(entries.contains(&"etc".to_string()));
        assert!(entries.contains(&"tmp".to_string()));
    }

    #[test]
    fn test_list_etc() {
        let fs = FakeFilesystem::new();
        let entries = fs.list_dir(Path::new("/etc"));
        assert!(entries.contains(&"passwd".to_string()));
        assert!(entries.contains(&"hosts".to_string()));
    }
}
