use std::sync::Arc;

use anyhow::Result;
use once_cell::sync::OnceCell;
use vfs::FileSystem;

// DynClone doesnt work since we're using a OnceCell (stores a reference, which we would need to then deref)
static FILESYSTEM: OnceCell<Box<dyn FileSystem>> = OnceCell::new();

pub(super) fn init_fs(fs: impl FileSystem) {
    let boxed_fs = Box::new(fs);
    FILESYSTEM.set(boxed_fs).unwrap();
}

pub(crate) fn get_fs<'a>() -> Result<&'a dyn FileSystem> {
    let fs = FILESYSTEM
        .get()
        .ok_or_else(|| anyhow::anyhow!("Filesystem not initialized"))?;

    let fs = fs.as_ref();
    Ok(fs)
}

#[cfg(test)]
mod test {
    use super::init_fs;
    use vfs::MemoryFS;

    #[test]
    fn test_fs() {
        init_fs(MemoryFS::new());
        let fs = super::get_fs().unwrap();
        fs.create_dir("/foo").unwrap();
        assert!(fs.exists("/foo").unwrap());
        init_fs(MemoryFS::new());
        let fs = super::get_fs().unwrap();
        assert!(fs.exists("/foo").unwrap());
        assert!(!fs.exists("/bar").unwrap());
    }
}
