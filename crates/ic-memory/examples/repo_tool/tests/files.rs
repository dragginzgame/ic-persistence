//! File-publication faults stay local to their real filesystem boundary.

use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum FileFault {
    ArchiveAfter,
    ArchiveBefore,
    ArchiveFifo,
    ArchiveRead,
    WriteBefore(PathBuf),
    WriteAfter(PathBuf),
}

#[derive(Default, Serialize)]
pub(super) struct FileEffects {
    pub fault: Option<FileFault>,
    pub archive_change: Option<PathBuf>,
}

impl FileEffects {
    pub(super) fn copy_verified(
        &mut self,
        source: &Path,
        path: &Path,
        expected: Sha256Digest,
        calls: &mut Vec<Vec<String>>,
    ) -> Result<()> {
        calls.push(vec!["stream-write".to_owned(), path.display().to_string()]);
        if let Some(source) = self.archive_change.take() {
            match self.fault.as_ref() {
                Some(FileFault::ArchiveRead) => {
                    fs::remove_file(&source)?;
                    fs::create_dir(source)?;
                }
                Some(FileFault::ArchiveFifo) => {
                    fs::remove_file(&source)?;
                    require(
                        Command::new("mkfifo").arg(source).status()?.success(),
                        "FIFO fixture creation failed",
                    )?;
                }
                _ => fs::write(source, b"changed after digest admission")?,
            }
        }
        if self.fault == Some(FileFault::ArchiveBefore) {
            return Err("fixture failure before archive publication".into());
        }
        Processes.copy_verified(source, path, expected)?;
        if self.fault == Some(FileFault::ArchiveAfter) {
            return Err("fixture failure after archive publication".into());
        }
        Ok(())
    }

    pub(super) fn write_bytes(
        &mut self,
        path: &Path,
        bytes: &[u8],
        calls: &mut Vec<Vec<String>>,
    ) -> Result<()> {
        let phase = match &self.fault {
            Some(FileFault::WriteBefore(target)) if path.ends_with(target) => Some(false),
            Some(FileFault::WriteAfter(target)) if path.ends_with(target) => Some(true),
            _ => None,
        };
        if phase.is_some() {
            self.fault = None;
        }
        if phase == Some(false) {
            return Err(std::io::Error::other("fixture failure before publication").into());
        }
        Processes.write_bytes(path, bytes)?;
        if phase == Some(true) {
            // A real parent-sync error can follow rename. Leave the published
            // bytes visible so recovery must inspect them before restoration.
            assert_eq!(fs::read(path)?, bytes);
            calls.push(vec![
                "published-write-error".to_owned(),
                path.display().to_string(),
            ]);
            return Err(std::io::Error::other("fixture failure after publication").into());
        }
        Ok(())
    }
}
