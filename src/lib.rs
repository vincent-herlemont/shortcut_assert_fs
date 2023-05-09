use assert_fs::fixture::FixtureError;
use assert_fs::TempDir;
use std::env::var_os;
use std::fs;
use std::fs::create_dir;
use std::io;
use std::time::{SystemTime, UNIX_EPOCH};
use filetime::FileTime;
use walkdir;
use walkdir::WalkDir;
use camino::{Utf8Path, Utf8PathBuf};
#[cfg(unix)]
use std::os::unix::fs::symlink;
#[cfg(windows)]
use std::os::windows::fs::{symlink_file, symlink_dir};

pub struct TmpFs {
    tmp_dir: TempDir,
}

impl TmpFs {
    pub fn new() -> Result<Self, FixtureError> {
        let tmp_dir = TempDir::new()
            .map(|tmp_dir| tmp_dir.into_persistent_if(var_os("TEST_PERSIST_FILES").is_some()))?;

        Ok(Self { tmp_dir })
    }

    fn tmp_path(&self) -> &Utf8Path {
        Utf8Path::from_path(self.tmp_dir.path()).unwrap()
    }

    #[allow(dead_code)]
    pub fn tmp_dir(&self) -> &TempDir {
        &self.tmp_dir
    }

    #[allow(dead_code)]
    pub fn path<PA: AsRef<Utf8Path>>(&self, path: PA) -> Utf8PathBuf {
        self.tmp_path().join(path)
    }


    #[allow(dead_code)]
    pub fn dir_entries_no_uf8(&self) -> Vec<std::path::PathBuf> {
        WalkDir::new(self.tmp_dir.path())
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.path().to_path_buf())
            .collect()
    }

    #[allow(dead_code)]
    pub fn display_dir_entries_no_uf8(&self) {
        for entry in self.dir_entries_no_uf8() {
            println!("{}", entry.to_string_lossy());
        }
    }

    #[allow(dead_code)]
    pub fn dir_entries(&self) -> Vec<Utf8PathBuf> {
        self.dir_entries_no_uf8()
            .iter()
            .map(|p| Utf8PathBuf::from_path_buf(p.to_path_buf()).unwrap())
            .collect()
    }

    #[allow(dead_code)]
    pub fn display_dir_entries(&self) {
        for entry in self.dir_entries() {
            println!("{}", entry.to_string());
        }
    }

    #[allow(dead_code)]
    pub fn write_file<PA: AsRef<Utf8Path>>(&self, path: PA, content: &str) -> io::Result<Utf8PathBuf> {
        let path = self.tmp_path().join(path);
        if let Some(path)  = path.parent() {
            self.create_dir_all(path)?;
        }
        fs::write(&path, content)?;
        Ok(path)
    }

    #[allow(dead_code)]
    pub fn rename<PA: AsRef<Utf8Path>>(&self, from: PA, to: PA) -> io::Result<()> {
        let from = self.tmp_path().join(from);
        let to = self.tmp_path().join(to);
        fs::rename(from, to)
    }

    #[allow(dead_code)]
    pub fn remove_file<PA: AsRef<Utf8Path>>(&self, path: PA) -> io::Result<()> {
        fs::remove_file(self.tmp_path().join(path))
    }

    #[allow(dead_code)]
    pub fn remove_dir_all<PA: AsRef<Utf8Path>>(&self, path: PA) -> io::Result<()> {
        fs::remove_dir_all(self.tmp_path().join(path))
    }

    #[allow(dead_code)]
    pub fn set_modification_time<PA: AsRef<Utf8Path>>(&self, path: PA) -> io::Result<()> {
        let now = SystemTime::now();
        let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
        let modified = FileTime::from_unix_time(since_the_epoch.as_secs() as i64, since_the_epoch.subsec_nanos());
        filetime::set_file_times(path.as_ref(), modified, modified)
    }

    #[allow(dead_code)]
    pub fn create_symbolic_link<PA: AsRef<Utf8Path>>(&self, from: PA, to: PA) -> io::Result<()> {
        let from = self.tmp_path().join(from);
        let to = self.tmp_path().join(to);
        #[cfg(unix)]
        symlink(from, to)?;
        #[cfg(windows)]
        symlink_file(from, to)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn read_file<PA: AsRef<Utf8Path>>(&self, path: PA) -> io::Result<Vec<u8>> {
        fs::read(self.tmp_path().join(path))
    }

    #[allow(dead_code)]
    pub fn replacen_file<PA: AsRef<Utf8Path>>(
        &self,
        file: PA,
        pat: &str,
        to: &str,
        count: usize,
    ) -> io::Result<()> {
        let path_file = self.tmp_path().join(file);
        let content = fs::read_to_string(&path_file)?.replacen(pat, to, count);
        fs::write(&path_file, content)
    }

    #[allow(dead_code)]
    pub fn create_dir_all<P: AsRef<Utf8Path>>(&self, path: P) -> io::Result<()> {
        fs::create_dir_all(self.tmp_path().join(path))
    }

    #[allow(dead_code)]
    pub fn copy_assets(&self, include_dir: &include_dir::Dir) -> io::Result<()> {
        copy(include_dir.entries(), &self.tmp_path())
    }
}


fn copy(source_entries: &[include_dir::DirEntry], path: &Utf8Path) -> io::Result<()> {
    for source_entry in source_entries {
        let source_path = source_entry.path().file_name().unwrap().to_str().unwrap();
        let target_path = path.join(source_path);
        match source_entry {
            include_dir::DirEntry::Dir(dir_entry) => {
                create_dir(&target_path)?;
                copy(dir_entry.entries(), &target_path)?;
            }
            include_dir::DirEntry::File(file_entry) => {
                fs::write(&target_path,file_entry.contents())?;
            }
        }
    }
    Ok(())
}