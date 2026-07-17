use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Atomically replaces `path` with a fully written, fsynced file.
///
/// The temporary file is created in the destination directory so the final
/// rename stays on one filesystem. On Unix, both the file and a newly created
/// destination directory are restricted to the current user.
pub fn atomic_write(path: &Path, contents: &[u8]) -> io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let parent_existed = parent.exists();
    fs::create_dir_all(parent)?;
    if !parent_existed {
        restrict_directory_permissions(parent)?;
    }

    let mut temporary = tempfile::Builder::new()
        .prefix(".atla-")
        .suffix(".tmp")
        .tempfile_in(parent)?;
    restrict_file_permissions(temporary.as_file())?;
    temporary.write_all(contents)?;
    temporary.flush()?;
    temporary.as_file().sync_all()?;

    let persisted = temporary.persist(path).map_err(|error| error.error)?;
    persisted.sync_all()?;
    sync_directory(parent)?;
    Ok(())
}

#[cfg(unix)]
fn restrict_file_permissions(file: &fs::File) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    file.set_permissions(fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn restrict_file_permissions(_file: &fs::File) -> io::Result<()> {
    Ok(())
}

#[cfg(unix)]
fn restrict_directory_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
fn restrict_directory_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> io::Result<()> {
    fs::File::open(path)?.sync_all()
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomically_replaces_existing_contents() {
        let directory = tempfile::tempdir().expect("temp directory");
        let path = directory.path().join("config.toml");
        fs::write(&path, b"old").expect("seed file");

        atomic_write(&path, b"new").expect("atomic replacement");

        assert_eq!(fs::read(&path).expect("read replacement"), b"new");
        let leftovers = fs::read_dir(directory.path())
            .expect("read directory")
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().starts_with(".atla-"))
            .count();
        assert_eq!(leftovers, 0);
    }

    #[cfg(unix)]
    #[test]
    fn writes_user_only_file_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempfile::tempdir().expect("temp directory");
        let path = directory.path().join("credentials.toml");

        atomic_write(&path, b"secret").expect("atomic write");

        let mode = fs::metadata(path).expect("metadata").permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
}
