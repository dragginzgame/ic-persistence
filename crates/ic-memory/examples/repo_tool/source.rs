//! Source responsibilities for the repository adapter.
use super::*;

impl<E: Execute> Repository<E> {
    pub(super) fn clean(&self) -> Result<()> {
        let status = self.output(
            "git",
            &[
                "--no-optional-locks",
                "status",
                "--porcelain",
                "--untracked-files=all",
            ],
        )?;
        require(
            status.trim().is_empty(),
            &format!(
                "working tree/index must be clean; review these Git status entries:\n{}",
                status.trim_end()
            ),
        )
    }

    pub(super) fn configuration(&self) -> Result<BTreeMap<String, String>> {
        let mut configuration = BTreeMap::new();
        for (name, value) in env::vars_os() {
            let name = name.to_str().ok_or("non-UTF-8 environment variable name")?;
            // The receipts identify the declared toolchains. Cargo compiler
            // replacements or wrappers would execute outside those identities.
            require(
                !is_compiler_override(name),
                &format!("unset {name} before release qualification; use the declared toolchains"),
            )?;
            if is_build_variable(name) {
                configuration.insert(
                    format!("env:{name}"),
                    value
                        .into_string()
                        .map_err(|_| "non-UTF-8 build environment")?,
                );
            }
        }
        let cargo_home = env::var_os("CARGO_HOME");
        let home = env::var_os("HOME");
        require(
            cargo_home.is_some() || home.is_some(),
            "set CARGO_HOME or HOME",
        )?;
        // Keep conservative custody of both root names, even when Cargo shadows
        // one. Build owns discovery, include traversal and same-read raw hashes.
        let roots: Vec<_> =
            configuration_directories(&self.root, cargo_home.as_deref(), home.as_deref())?
                .iter()
                .flat_map(|directory| [directory.join("config"), directory.join("config.toml")])
                .filter(|path| path.exists())
                .collect();
        for file in observe_configurations(&roots, CONFIGURATION_LIMITS)? {
            if let Some(field) = file.compiler_overrides.first() {
                return Err(format!(
                    "remove build.{field} from {} before release qualification; use the declared toolchains",
                    file.path.display()
                ).into());
            }
            // Contents may contain secrets; receipts retain only lookup labels
            // and the hash of the exact bytes parsed by the observer.
            configuration.insert(
                format!("config:{}", file.path.display()),
                file.sha256.to_string(),
            );
        }
        Ok(configuration)
    }

    pub(super) fn check_selection(&self, selection: &ReleaseSelection) -> Result<()> {
        self.version_parts(&selection.previous)?;
        self.version_parts(&selection.version)?;
        self.validate_date(&selection.date)?;
        require(
            self.output(
                "bash",
                &[
                    "scripts/ci/next-release-version.sh",
                    &selection.previous,
                    &selection.kind,
                ],
            )?
            .trim()
                == selection.version,
            "candidate differs from the common version selection",
        )?;
        require(
            self.version(Some(&selection.source))? == selection.previous,
            "saved source version differs from the release selection",
        )
    }

    pub(super) fn member_release_manifest(
        &self,
        revision: Option<&str>,
        name: &str,
    ) -> Result<String> {
        let text = self.text(&package_manifest(name), revision)?;
        let manifest: toml::Value = toml::from_str(&text)?;
        let package = manifest
            .get("package")
            .ok_or("missing Persistence package")?;
        require(
            package.get("name").and_then(toml::Value::as_str) == Some(name),
            "unexpected Persistence package",
        )?;
        require(
            package
                .get("version")
                .and_then(|version| version.get("workspace"))
                .and_then(toml::Value::as_bool)
                == Some(true),
            "expected inherited Persistence version",
        )?;
        Ok(text)
    }

    pub(super) fn surfaces(
        &self,
        selection: &ReleaseSelection,
    ) -> Result<BTreeMap<String, ReleaseSurface>> {
        self.check_selection(selection)?;
        let source = &selection.source;
        let previous = &selection.previous;
        let version = &selection.version;
        let [major, minor, _] = self.version_parts(version)?;
        let detail = format!("docs/changelog/{major}.{minor}.md");
        let finalizer = self.root.join(CHANGELOG_FINALIZER);
        let mut surfaces = BTreeMap::new();
        for path in [
            "Cargo.lock",
            "Cargo.toml",
            PACKAGE_README,
            "CHANGELOG.md",
            &detail,
        ] {
            let original = self.text(path, Some(source))?;
            let expected = match path {
                "Cargo.lock" => Self::replace_lock_version(&original, previous, version)?,
                "Cargo.toml" => self.replace_workspace_version(&original, previous, version)?,
                PACKAGE_README => original.clone(),
                _ => self.release_changelog(
                    &finalizer,
                    &original,
                    previous,
                    version,
                    &selection.date,
                )?,
            };
            surfaces.insert(path.to_owned(), ReleaseSurface { original, expected });
        }
        for name in packages().into_iter().skip(1) {
            let original = self.member_release_manifest(Some(source), name)?;
            surfaces.insert(
                package_manifest(name),
                ReleaseSurface {
                    expected: original.clone(),
                    original,
                },
            );
        }
        Ok(surfaces)
    }

    pub(super) fn check_surfaces(
        &self,
        selection: &ReleaseSelection,
        surfaces: &BTreeMap<String, ReleaseSurface>,
        revision: Option<&str>,
        partial: bool,
    ) -> Result<()> {
        let mut expected_changes = Vec::new();
        for (path, surface) in surfaces {
            let actual = self.text(path, revision)?;
            require(
                actual == surface.expected || partial && actual == surface.original,
                &format!(
                    "{path} does not match the admitted release bytes for source {}. \
                     For a new release, commit intended source edits first; git add alone is insufficient. \
                     For recovery, preserve the saved plan and reconcile the conflicting file",
                    selection.source
                ),
            )?;
            if actual != surface.original {
                expected_changes.push(path.clone());
            }
        }
        let mut args = vec!["diff", "--name-only", &selection.source];
        if let Some(revision) = revision {
            args.push(revision);
        }
        args.push("--");
        require(
            self.output("git", &args)?
                .lines()
                .eq(expected_changes.iter().map(String::as_str)),
            "release candidate contains changes outside its version surfaces",
        )?;
        require(
            self.output("git", &["ls-files", "--others", "--exclude-standard"])?
                .is_empty(),
            "release candidate has untracked files",
        )
    }

    pub(super) fn check_index(
        &self,
        selection: &ReleaseSelection,
        surfaces: &BTreeMap<String, ReleaseSurface>,
    ) -> Result<()> {
        for path in self
            .output(
                "git",
                &["diff", "--cached", "--name-only", &selection.source, "--"],
            )?
            .lines()
        {
            let expected = surfaces
                .get(path)
                .ok_or("index contains unrelated release changes")?;
            require(
                self.git(&[
                    "diff",
                    "--cached",
                    "--summary",
                    &selection.source,
                    "--",
                    path,
                ])?
                .is_empty(),
                "staged release metadata changes file mode or identity",
            )?;
            let actual = self.output("git", &["show", &format!(":{path}")])?;
            require(
                actual == expected.expected || actual == expected.original,
                "staged release metadata differs from saved intent",
            )?;
        }
        Ok(())
    }

    pub(super) fn text(&self, path: &str, revision: Option<&str>) -> Result<String> {
        match revision {
            Some(revision) => self.output("git", &["show", &format!("{revision}:{path}")]),
            None => Ok(fs::read_to_string(self.root.join(path))?),
        }
    }
}
