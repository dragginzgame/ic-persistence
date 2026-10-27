//! Cargo and Make substitutes retain their own build failures.
use super::*;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum BuildFault {
    DependencyUpdate,
    Formatting,
    MakeExecution,
    Metadata,
    Package,
    PackageHead,
    RelativeMetadata,
    ToolCheck,
    ToolSetup,
    Validate,
    ValidationLock,
}

#[derive(Default, Deserialize, Serialize)]
pub(super) struct BuildEffects {
    pub fail: Option<BuildFault>,
}

impl BuildEffects {
    pub(super) fn target(&self, root: &Path) -> Result<PathBuf> {
        match self.fail.as_ref() {
            Some(BuildFault::Metadata) => Err("fixture metadata failure".into()),
            Some(BuildFault::RelativeMetadata) => Ok(PathBuf::from("relative output")),
            _ => Ok(root.join("configured output")),
        }
    }

    pub(super) fn capture(
        &self,
        root: &Path,
        program: &str,
        args: &[&str],
        git: &mut GitState,
    ) -> Result<String> {
        match program {
            "rustc" => Ok(format!("fixture-rustc {}\n", args[0])),
            "cargo" if args.get(1) == Some(&"--version") => {
                Ok(format!("fixture-cargo {}\n", args[0]))
            }
            "cargo" if args.get(1) == Some(&"build") => Self::probe_build(root, args),
            "cargo" if args.get(1) == Some(&"update") => self.update(root),
            "cargo" if args.get(1) == Some(&"fetch") => Ok(String::new()),
            "cargo" | "bash"
                if args.get(1) == Some(&"package")
                    || args.first() == Some(&"scripts/ci/package-workspace.sh") =>
            {
                self.package(root, git)
            }
            "bash" if args == ["scripts/ci/check-make-execution.sh"] => {
                self.gate(BuildFault::MakeExecution, "fixture Make execution refusal")
            }
            "make" if args == ["--no-print-directory", "install-tools"] => {
                self.gate(BuildFault::ToolSetup, "fixture tool setup failure")
            }
            "make" if args == ["--no-print-directory", "tools-check", "format-tools-check"] => self
                .gate(
                    BuildFault::ToolCheck,
                    "fixture offline tool admission failure",
                ),
            "make"
                if args
                    == [
                        "--no-print-directory",
                        "validate",
                        "VALIDATION_TOOLCHAIN=1.99.0",
                    ] =>
            {
                self.gate(BuildFault::Validate, "fixture validation failure")?;
                if self.fail == Some(BuildFault::ValidationLock) {
                    fs::write(root.join("Cargo.lock"), "unexpected dependency selection")?;
                }
                Ok(String::new())
            }
            "make"
                if args
                    == [
                        "--no-print-directory",
                        "fmt-check",
                        "VALIDATION_TOOLCHAIN=1.99.0",
                    ] =>
            {
                self.gate(BuildFault::Formatting, "fixture formatting failure")
            }
            _ => Err(format!("unexpected fixture process: {program} {args:?}").into()),
        }
    }

    fn gate(&self, fault: BuildFault, message: &str) -> Result<String> {
        if self.fail == Some(fault) {
            return Err(message.into());
        }
        Ok(String::new())
    }
    fn probe_build(root: &Path, args: &[&str]) -> Result<String> {
        require(
            args[2..8]
                == [
                    "--locked",
                    "--offline",
                    "--profile",
                    "wasm-size",
                    "--target",
                    "wasm32-unknown-unknown",
                ],
            "unexpected probe build options",
        )?;
        // Artifacts already supplied by the budget fixture. Only admit
        // selectors with matching files, so omitting the build cannot pass.
        for selector in args[8..].chunks(2) {
            require(
                selector.len() == 2 && selector[0] == "--example",
                "unexpected probe selector",
            )?;
            require(
                root.join("configured output/wasm32-unknown-unknown/wasm-size/examples")
                    .join(format!("{}.wasm", selector[1].replace('-', "_")))
                    .is_file(),
                "probe selector has no matching artifact",
            )?;
        }
        Ok(String::new())
    }

    pub(super) fn version(root: &Path) -> Result<String> {
        toml_string(
            &fs::read_to_string(root.join("Cargo.toml"))?,
            "workspace.package",
            "version",
        )
    }
    fn update(&self, root: &Path) -> Result<String> {
        let mut lock = fs::read_to_string(root.join("Cargo.lock"))?.replacen(
            "name = \"ic-memory\"\nversion = \"0.12.3\"",
            &format!(
                "name = \"ic-memory\"\nversion = \"{}\"",
                Self::version(root)?
            ),
            1,
        );
        if self.fail.as_ref() == Some(&BuildFault::DependencyUpdate) {
            lock = lock.replace(
                "name = \"other\"\nversion = \"0.12.3\"",
                "name = \"other\"\nversion = \"0.12.4\"",
            );
        }
        fs::write(root.join("Cargo.lock"), lock)?;
        Ok(String::new())
    }
    fn package(&self, root: &Path, git: &mut GitState) -> Result<String> {
        let target = root.join("configured output");

        let directory = target.join("package");
        fs::create_dir_all(&directory)?;
        if self.fail.as_ref() == Some(&BuildFault::PackageHead) {
            let mut descendant = git.release.clone().ok_or("missing release")?;
            descendant.insert(
                "source.rs".to_owned(),
                "// fix during packaging\n".to_owned(),
            );
            for (path, text) in &descendant {
                fs::write(root.join(path), text)?;
            }
            git.descendant = Some(descendant);
        }
        // Model Cargo's Git metadata changing after the release commit.
        let bytes = if git.descendant.is_some() {
            "descendant-archive"
        } else if git.release.is_some() {
            "release-archive"
        } else {
            "prepared-archive"
        };
        fs::write(
            directory.join(format!("ic-memory-{}.crate", Self::version(root)?)),
            bytes,
        )?;
        fs::write(
            directory.join(format!("ic-storage-{}.crate", Self::version(root)?)),
            format!("storage-{bytes}"),
        )?;
        fs::write(
            directory.join(format!("ic-metrics-stable-{}.crate", Self::version(root)?)),
            format!("metrics-{bytes}"),
        )?;
        if self.fail.as_ref() == Some(&BuildFault::Package) {
            return Err("fixture package failure".into());
        }
        Ok(String::new())
    }
}
