//! Source-derived metadata publication and owned rollback.
use super::*;

///
/// PreparationError
///
/// Original preparation failure together with failed metadata restorations.
///
/// Successful restorations and retained evidence remain available for recovery.
///

#[derive(Debug)]
pub(super) struct PreparationError {
    pub(super) source: Box<dyn Error>,
    pub(super) rollback_errors: Vec<(String, Box<dyn Error>)>,
}

impl std::fmt::Display for PreparationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.source)?;
        for (path, error) in &self.rollback_errors {
            write!(formatter, "; could not restore {path}: {error}")?;
        }
        Ok(())
    }
}

impl Error for PreparationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

// Each edit keeps rollback bytes beside the exact source-derived replacement.
struct ReleaseEdit<'a> {
    path: &'a str,
    surface: &'a ReleaseSurface,
    before: Vec<u8>,
}

impl ReleaseEdit<'_> {
    fn publish<E: Execute>(&self, repo: &Repository<E>) -> Result<()> {
        if self.before != self.surface.expected.as_bytes() {
            repo.exec
                .write_bytes(&repo.root.join(self.path), self.surface.expected.as_bytes())?;
        }
        Ok(())
    }

    fn restore<E: Execute>(
        &self,
        repo: &Repository<E>,
        selection: &ReleaseSelection,
    ) -> Result<()> {
        let current = fs::read(repo.root.join(self.path))?;
        if current == self.before {
            // Preserve file identity and permissions when no replacement needs undoing.
            return Ok(());
        }
        let owned = if self.path == "Cargo.lock" {
            std::str::from_utf8(&current).is_ok_and(|text| {
                check_lock_update(&self.before, text, &selection.previous, &selection.version)
                    .is_ok()
            })
        } else {
            current == self.surface.expected.as_bytes()
        };
        if owned {
            repo.exec
                .write_bytes(&repo.root.join(self.path), &self.before)?;
        }
        Ok(())
    }
}

// The immutable source projection and live rollback observation remain distinct.
pub(super) struct ReleasePlan<'a> {
    edits: Vec<ReleaseEdit<'a>>,
}

impl<'a> ReleasePlan<'a> {
    pub(super) fn capture<E: Execute>(
        repo: &Repository<E>,
        surfaces: &'a BTreeMap<String, ReleaseSurface>,
    ) -> Result<Self> {
        let edits = surfaces
            .iter()
            .map(|(path, surface)| {
                Ok(ReleaseEdit {
                    path,
                    surface,
                    before: fs::read(repo.root.join(path))?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { edits })
    }

    pub(super) fn publish<E: Execute>(&self, repo: &Repository<E>) -> Result<()> {
        // The root version is the completion marker for the complete metadata projection.
        for edit in self
            .edits
            .iter()
            .filter(|edit| edit.path != "Cargo.toml")
            .chain(self.edits.iter().filter(|edit| edit.path == "Cargo.toml"))
        {
            edit.publish(repo)?;
        }
        Ok(())
    }

    pub(super) fn rollback<E: Execute>(
        &self,
        repo: &Repository<E>,
        selection: &ReleaseSelection,
        source: Box<dyn Error>,
    ) -> Box<dyn Error> {
        // Restore every owned edit even when another restoration fails.
        let mut rollback_errors = Vec::new();
        for edit in &self.edits {
            if let Err(error) = edit.restore(repo, selection) {
                rollback_errors.push((edit.path.to_owned(), error));
            }
        }
        if rollback_errors.is_empty() {
            source
        } else {
            Box::new(PreparationError {
                source,
                rollback_errors,
            })
        }
    }
}
