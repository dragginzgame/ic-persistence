//! Shared disposable repository and substituted release effects.

use super::*;

pub(super) const PACKAGE_MANIFEST: &str = "crates/ic-memory/Cargo.toml";

// Only these narrow command shapes are semantic effects in this fixture.
// Keep raw argv logs and exact-dispatch assertions independent of these predicates.
pub(super) fn is_cargo_call(call: &[String], command: &str) -> bool {
    call.first().is_some_and(|program| program == "cargo")
        && call.get(2).is_some_and(|argument| argument == command)
}

pub(super) fn is_make_call(call: &[String], target: &str) -> bool {
    call.first().is_some_and(|program| program == "make")
        && call
            .get(1)
            .is_some_and(|argument| argument == "--no-print-directory")
        && call.get(2).is_some_and(|argument| argument == target)
}

pub(super) fn is_package_call(call: &[String]) -> bool {
    is_cargo_call(call, "package")
        || call.first().is_some_and(|program| program == "bash")
            && call
                .get(1)
                .is_some_and(|script| script == "scripts/ci/package-workspace.sh")
}

pub(super) const MANIFEST: &str = "[workspace]\nmembers = [\"crates/ic-memory\", \"crates/ic-storage\", \"crates/ic-metrics-stable\", \"crates/ic-logging-stable\"]\n[workspace.package]\nrust-version = \"1.88.0\"\n[workspace.dependencies]\nic-memory = { path = \"crates/ic-memory\", version = \"0.12.3\" }\nic-storage = { path = \"crates/ic-storage\", version = \"0.12.3\" }\nother = \"0.12.3\"\n";
pub(super) const PACKAGE: &str = "[package]\nname = \"ic-memory\"\nversion = \"0.12.3\"\n";
pub(super) const LOCK: &str = "version = 4\n[[package]]\nname = \"ic-logging-stable\"\nversion = \"0.12.3\"\n[[package]]\nname = \"ic-metrics-stable\"\nversion = \"0.12.3\"\ndependencies = [\"ic-memory\", \"ic-storage\"]\n[[package]]\nname = \"ic-storage\"\nversion = \"0.12.3\"\ndependencies = [\"ic-memory\"]\n[[package]]\nname = \"ic-memory\"\nversion = \"0.12.3\"\n[[package]]\nname = \"other\"\nversion = \"0.12.3\"\n";

pub(super) struct Fixture {
    pub(super) repo: Repository<Substitute>,
    pub(super) selection: ReleaseSelection,
}

impl Fixture {
    pub(super) fn new() -> Self {
        Self::new_in(&env::temp_dir())
    }

    pub(super) fn for_group(group: &str) -> Self {
        let mut fixture = Self::new();
        fixture.repo.group = group.to_owned();
        fixture.selection.group = group.to_owned();
        fixture.repo.exec.state.borrow_mut().git.group = group.to_owned();
        fixture
    }

    pub(super) fn new_in(parent: &Path) -> Self {
        // Exclusive creation owns uniqueness, including across PID reuse.
        // Never reuse or remove an occupied path: it may hold failed evidence.
        let mut sequence = 0_u64;
        let root = loop {
            let candidate = parent.join(format!(
                "ic-memory-tooling-{}-{sequence}",
                std::process::id()
            ));
            match fs::create_dir(&candidate) {
                Ok(()) => break candidate,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => sequence += 1,
                Err(error) => panic!("could not create fixture {}: {error}", candidate.display()),
            }
        };
        let mut source = BTreeMap::from([
            (".gitignore".to_owned(), "target/\n".to_owned()),
            (
                "crates/ic-memory/src/lib.rs".to_owned(),
                "// fixture library\n".to_owned(),
            ),
            ("Cargo.lock".to_owned(), LOCK.to_owned()),
            ("Cargo.toml".to_owned(), MANIFEST.to_owned()),
            (PACKAGE_MANIFEST.to_owned(), PACKAGE.to_owned()),
            (
                PACKAGE_README.to_owned(),
                "# Fixture\n\nic-memory = \"0.12.3\"\n".to_owned(),
            ),
            (
                "releases/persistence/CHANGELOG.md".to_owned(),
                "# Changelog\n\n## [0.13.0]\n\nFixture changes.\n\n## 0.12.3\n\nHistory.\n"
                    .to_owned(),
            ),
            ("source.rs".to_owned(), "// committed source\n".to_owned()),
            (
                "releases/persistence/0.13.md".to_owned(),
                "# Detail\n\n## [0.13.0]\n\nFixture changes.\n".to_owned(),
            ),
        ]);
        Self::group_sources(&mut source);
        for helper in [
            CHANGELOG_FINALIZER,
            RELEASE_INPUT_ADMISSION,
            "scripts/ci/StableVersion.pm",
        ] {
            source.insert(
                helper.to_owned(),
                fs::read_to_string(
                    Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../..")
                        .join(helper),
                )
                .unwrap(),
            );
        }
        for (path, text) in &source {
            fs::create_dir_all(root.join(path).parent().unwrap()).unwrap();
            fs::write(root.join(path), text).unwrap();
        }
        fs::write(
            root.join("rust-toolchain.toml"),
            "[toolchain]\nchannel = \"1.99.0\"\n",
        )
        .unwrap();
        let exec = Substitute {
            files: RefCell::new(FileEffects::default()),
            state: RefCell::new(State {
                git: GitState {
                    group: "persistence".to_owned(),
                    source,
                    ..GitState::default()
                },
                ..State::default()
            }),
        };
        Self {
            repo: Repository {
                root,
                exec,
                group: "persistence".to_owned(),
            },
            selection: ReleaseSelection {
                group: "persistence".to_owned(),
                kind: "minor".to_owned(),
                previous: "0.12.3".to_owned(),
                version: "0.13.0".to_owned(),
                date: "2026-10-05".to_owned(),
                source: "source".to_owned(),
                remote: "origin".to_owned(),
                branch: "main".to_owned(),
            },
        }
    }

    fn group_sources(source: &mut BTreeMap<String, String>) {
        for (name, library) in packages().into_iter().skip(1).zip([
            "// storage fixture\n",
            "// metrics fixture\n",
            "// logging fixture\n",
        ]) {
            source.insert(
                package_manifest(name),
                format!("[package]\nname = {name:?}\nversion = \"0.12.3\"\n"),
            );
            source.insert(format!("crates/{name}/src/lib.rs"), library.to_owned());
        }
        let policy_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ci/cargo-release-groups.json");
        let mut policy: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(policy_path).unwrap()).unwrap();
        for group in policy["groups"].as_array_mut().unwrap() {
            if group["name"] == "persistence" {
                group["metadata_files"] = serde_json::json!(["releases/persistence/0.13.md"]);
            }
        }
        source.insert(
            GROUP_POLICY.to_owned(),
            serde_json::to_string(&policy).unwrap(),
        );
        for name in ["ic-metrics-stable", "ic-logging-stable"] {
            source.insert(
                format!(
                    "releases/{}/CHANGELOG.md",
                    name.strip_prefix("ic-").unwrap()
                ),
                "# Adapter\n\n## [0.13.0]\n\nFixture changes.\n\n## [0.12.3] - 2026-10-04\n"
                    .to_owned(),
            );
        }
    }

    pub(super) fn select_detail(&self, path: &str) {
        let mut policy: serde_json::Value =
            serde_json::from_str(&self.repo.text(GROUP_POLICY, Some("source")).unwrap()).unwrap();
        policy["groups"][0]["metadata_files"] = serde_json::json!([path]);
        self.set_source(
            GROUP_POLICY,
            &serde_json::to_string_pretty(&policy).unwrap(),
        );
    }

    pub(super) fn package_evidence(&self, stage: ReceiptStage) -> PackageEvidence {
        let path = self
            .repo
            .receipt_path(&self.selection.version, stage)
            .unwrap();
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    }

    pub(super) fn validation_evidence(&self) -> ValidationEvidence {
        let path = self.repo.validation_path(&self.selection.version).unwrap();
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    }

    /// Set both the saved source projection and live bytes before qualification.
    /// Intentional source/worktree mismatches must keep their separate writes.
    pub(super) fn set_source(&self, path: &str, text: &str) {
        let file = self.repo.root.join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, text).unwrap();
        self.repo
            .exec
            .state
            .borrow_mut()
            .git
            .source
            .insert(path.to_owned(), text.to_owned());
    }

    pub(super) fn prepare(&self) -> Result<()> {
        self.repo.preflight(&self.selection)?;
        self.repo.verify(&self.selection)?;
        self.repo.preflight(&self.selection)?;
        self.repo.prepare_selected(&self.selection)
    }

    pub(super) fn stage(&self) -> Result<()> {
        self.repo.prepared(&self.selection)?;
        self.repo.exec.state.borrow_mut().git.staged = true;
        Ok(())
    }

    pub(super) fn commit(&self) -> Result<()> {
        if self.repo.exec.state.borrow().git.release.is_none() {
            self.repo.commit_check(&self.selection)?;
            let mut state = self.repo.exec.state.borrow_mut();
            let release = state
                .git
                .source
                .keys()
                .map(|path| {
                    (
                        path.clone(),
                        fs::read_to_string(self.repo.root.join(path)).unwrap(),
                    )
                })
                .collect();
            state.git.release = Some(release);
            state
                .calls
                .push(vec!["shared-runner".to_owned(), "commit".to_owned()]);
        }
        self.repo
            .selected_committed_check(&self.selection, "release")?;
        self.repo.exec.state.borrow_mut().git.tag = true;
        Ok(())
    }

    pub(super) fn release(&self) {
        self.prepare().unwrap();
        self.stage().unwrap();
        self.commit().unwrap();
    }

    pub(super) fn fail_build(&self, fault: BuildFault) {
        let mut state = self.repo.exec.state.borrow_mut();
        state.clear_faults();
        state.build.fail = Some(fault);
    }

    pub(super) fn fail_git(&self, fault: GitFault) {
        let mut state = self.repo.exec.state.borrow_mut();
        state.clear_faults();
        state.git.fail = Some(fault);
    }

    pub(super) fn fail_registry(&self, fault: RegistryFault) {
        let mut state = self.repo.exec.state.borrow_mut();
        state.clear_faults();
        state.registry.fail = Some(fault);
    }

    pub(super) fn fail_files(&self, fault: FileFault) {
        self.repo.exec.files.borrow_mut().fault = Some(fault);
    }

    pub(super) fn assert_original(&self) {
        assert_eq!(
            fs::read_to_string(self.repo.root.join("Cargo.toml")).unwrap(),
            MANIFEST
        );
        assert_eq!(
            fs::read_to_string(self.repo.root.join("Cargo.lock")).unwrap(),
            LOCK
        );
        assert!(self.repo.exec.state.borrow().git.release.is_none());
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if std::thread::panicking() {
            let saved = (|| -> Result<()> {
                let state = self.repo.exec.state.try_borrow()?;
                fs::write(
                    self.repo.root.join("fixture-state.json"),
                    serde_json::to_vec_pretty(&*state)?,
                )?;
                fs::write(
                    self.repo.root.join("fixture-files.json"),
                    serde_json::to_vec_pretty(&*self.repo.exec.files.try_borrow()?)?,
                )?;
                Ok(())
            })();
            if let Err(error) = saved {
                eprintln!("Could not save fixture command state: {error}");
            }
            eprintln!("Release fixture retained: {}", self.repo.root.display());
            return;
        }
        fs::remove_dir_all(&self.repo.root).unwrap();
    }
}

#[derive(Default, Deserialize, Serialize)]
pub(super) struct State {
    pub(super) git: GitState,
    pub(super) build: BuildEffects,
    pub(super) registry: RegistryEffects,
    pub(super) calls: Vec<Vec<String>>,
}

impl State {
    pub(super) fn has_call(&self, mut predicate: impl FnMut(&[String]) -> bool) -> bool {
        self.calls.iter().any(|call| predicate(call))
    }

    pub(super) fn call_position(&self, mut predicate: impl FnMut(&[String]) -> bool) -> usize {
        self.calls
            .iter()
            .position(|call| predicate(call))
            .expect("expected command was recorded")
    }

    pub(super) fn clear_faults(&mut self) {
        self.git.fail = None;
        self.build.fail = None;
        self.registry.fail = None;
    }
}

pub(super) struct Substitute {
    pub(super) state: RefCell<State>,
    pub(super) files: RefCell<FileEffects>,
}

impl Execute for Substitute {
    fn cargo_target(&self, root: &Path) -> Result<PathBuf> {
        let mut state = self.state.borrow_mut();
        state.calls.push(vec!["cargo-metadata".to_owned()]);
        state.build.target(root)
    }

    fn write_json(&self, path: &Path, value: &impl Serialize) -> Result<()> {
        // Reuse the fixture's before/after-publication fault injection. The
        // native streaming boundary is exercised separately below.
        self.write_bytes(path, &serde_json::to_vec_pretty(value)?)
    }

    fn copy_verified(&self, source: &Path, path: &Path, expected: Sha256Digest) -> Result<()> {
        self.files.borrow_mut().copy_verified(
            source,
            path,
            expected,
            &mut self.state.borrow_mut().calls,
        )
    }

    fn write_bytes(&self, path: &Path, bytes: &[u8]) -> Result<()> {
        self.files
            .borrow_mut()
            .write_bytes(path, bytes, &mut self.state.borrow_mut().calls)
    }

    fn execute(&self, root: &Path, program: &str, args: &[&str]) -> Result<()> {
        self.capture(root, program, args).map(|_| ())
    }

    fn capture(&self, root: &Path, program: &str, args: &[&str]) -> Result<String> {
        if program == "env" {
            return self.publication(root, args);
        }
        let mut state = self.state.borrow_mut();
        state.calls.push(
            std::iter::once(program)
                .chain(args.iter().copied())
                .map(str::to_owned)
                .collect(),
        );
        let State {
            git,
            build,
            registry,
            calls,
        } = &mut *state;
        match program {
            "git" => git.capture(root, args, calls),
            "bash" if args.first() == Some(&TAG_CHECKER) => git.check_tag(args),
            "curl" => registry.observe(root, args),
            "cargo" if args.get(1) == Some(&"publish") => registry.publish(root, args, git),
            "perl" | "awk" => Processes.capture(root, program, args),
            "bash"
                if args == ["scripts/ci/check-make-execution.sh"]
                    && build.fail == Some(BuildFault::MakeExecution) =>
            {
                build.capture(root, program, args, git)
            }
            "bash" if args.first() != Some(&"scripts/ci/package-workspace.sh") => Processes
                .capture(
                    &Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
                    program,
                    args,
                ),
            _ => build.capture(root, program, args, git),
        }
    }
}

impl Substitute {
    pub(super) fn publication(&self, root: &Path, args: &[&str]) -> Result<String> {
        require(
            args.len() == 10
                && args[0] == "RUSTUP_TOOLCHAIN=1.99.0"
                && args[1] == "bash"
                && args[2] == "scripts/ci/run-cargo-publication.sh"
                && matches!(args[3], "check" | "publish")
                && args[5] == "release"
                && args[6] == "crates-io"
                && args[7] == "--package"
                && args[9] == "--no-verify",
            "unexpected coordinated publication attempt",
        )?;
        let (name, version) = args[8].split_once('@').ok_or("missing exact selector")?;
        require(
            version == BuildEffects::package_version(root, name)?,
            "wrong publication version",
        )?;
        let mut forwarded = vec![
            "+1.99.0",
            "publish",
            "--locked",
            "--registry",
            "crates-io",
            "--package",
            name,
            "--no-verify",
        ];
        if args[3] == "check" {
            forwarded.push("--dry-run");
        }
        self.capture(root, "cargo", &forwarded)
    }
}

/// Bytes captured before a refused effect; file identity checks remain explicit.
pub(super) struct FileSnapshot {
    path: PathBuf,
    bytes: Vec<u8>,
}
impl FileSnapshot {
    pub(super) fn capture(path: &Path) -> Self {
        Self {
            path: path.to_owned(),
            bytes: fs::read(path).unwrap(),
        }
    }
    pub(super) const fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }
    pub(super) fn assert_unchanged(&self) {
        assert_eq!(
            fs::read(&self.path).unwrap(),
            self.bytes,
            "changed file: {}",
            self.path.display()
        );
    }
}
