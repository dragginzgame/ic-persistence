//! Consumer check roster over Build's admitted, immutable archive workspace.
use super::*;
use ic_build::cargo::{
    observation::decode_metadata,
    packages::{
        PackageArchiveSpec, PackageQualificationLimits, capture_package_command,
        qualify_existing_package_archives, verify_external_lock_subset,
    },
    selection::CargoFeatureSelection,
};
use ic_host_process::tool::{OutputLimit, OutputLimits};
use std::{fmt::Write as _, process::Command};

const FILE_BYTES: usize = 16 * 1024 * 1024;
const PROCESS_LIMITS: OutputLimits = OutputLimits {
    stdout: OutputLimit::Terminate(8 * 1024 * 1024),
    stderr: OutputLimit::Terminate(1024 * 1024),
    timeout: None,
};

impl<E: Execute> Repository<E> {
    pub(super) fn qualify_archives(&self, evidence: &Path, msrv: &str) -> Result<()> {
        let toolchain = self.toolchain()?;
        require(
            msrv == self.msrv()? && msrv == toolchain,
            "archive compiler must match the pinned support floor",
        )?;
        let metadata = self.output(
            "cargo",
            &[
                &format!("+{toolchain}"),
                "metadata",
                "--locked",
                "--offline",
                "--no-deps",
                "--format-version",
                "1",
            ],
        )?;
        let metadata = decode_metadata(metadata.as_bytes())?;
        let mut selected = Vec::new();
        for group in self.release_groups(None)? {
            for name in group.members {
                let package = metadata
                    .packages
                    .iter()
                    .find(|package| {
                        package.name.as_str() == name
                            && metadata.workspace_members.contains(&package.id)
                    })
                    .ok_or("archive selection is not a workspace member")?;
                selected.push((name, package.version.to_string()));
            }
        }
        let specs = selected
            .iter()
            .map(|(name, version)| PackageArchiveSpec {
                name,
                version,
                patch_registry: true,
            })
            .collect::<Vec<_>>();
        let evidence = evidence.canonicalize()?;
        qualify_existing_package_archives(
            &self.root,
            &metadata
                .target_directory
                .join("package")
                .into_std_path_buf(),
            &evidence,
            &specs,
            &CargoFeatureSelection::default(),
            PackageQualificationLimits {
                process: PROCESS_LIMITS,
                file_bytes: FILE_BYTES,
                source_entries: 16_384,
                source_depth: 32,
            },
            || cargo(&toolchain),
            |archive, directory| {
                let mut command = Command::new("tar");
                command.arg("-xzf").arg(archive).arg("-C").arg(directory);
                command
            },
            |workspace| self.qualify_archive_consumer(workspace, &selected, &toolchain, msrv),
        )?;
        Ok(())
    }

    fn qualify_archive_consumer(
        &self,
        workspace: &Path,
        packages: &[(String, String)],
        toolchain: &str,
        msrv: &str,
    ) -> Result<()> {
        // Build has already admitted each extracted package and the exact frozen
        // graph. Keep this consumer-specific roster, without changing its manifest.
        for (index, (name, version)) in packages.iter().enumerate() {
            let mut command = cargo(toolchain);
            command
                .args(["check", "--locked", "--offline", "--manifest-path"])
                .arg(workspace.join(format!("{name}-{version}/Cargo.toml")))
                .arg("--target-dir")
                .arg(workspace.join("target"));
            check(&mut command, workspace, &format!("library-{index}"))?;
        }
        let consumer = workspace.join("consumer");
        fs::create_dir_all(consumer.join("src"))?;
        fs::copy(
            self.root.join("testing/package-consumer.rs"),
            consumer.join("src/main.rs"),
        )?;
        let manifest = consumer_manifest(workspace, packages, msrv)?;
        let manifest_path = consumer.join("Cargo.toml");
        fs::write(&manifest_path, &manifest)?;
        let source = fs::read(consumer.join("src/main.rs"))?;
        let selected_lock = fs::read(workspace.join("Cargo.lock"))?;
        fs::write(consumer.join("Cargo.lock"), &selected_lock)?;
        let mut command = cargo(toolchain);
        command
            .args([
                "metadata",
                "--offline",
                "--format-version",
                "1",
                "--manifest-path",
            ])
            .arg(&manifest_path);
        let metadata = check(&mut command, workspace, "consumer-metadata")?;
        let metadata = decode_metadata(&metadata)?;
        let lock = fs::read(consumer.join("Cargo.lock"))?;
        verify_external_lock_subset(&selected_lock, &lock, FILE_BYTES)?;
        for (name, version) in packages {
            let expected = workspace.join(format!("{name}-{version}/Cargo.toml"));
            let matches = metadata
                .packages
                .iter()
                .filter(|package| package.name.as_str() == name)
                .collect::<Vec<_>>();
            require(
                matches.len() == 1
                    && matches[0].source.is_none()
                    && matches[0].manifest_path.as_std_path() == expected,
                "consumer substituted an extracted package",
            )?;
        }
        require(
            metadata.packages.iter().all(|package| {
                package.source.is_some()
                    || package.manifest_path.as_std_path() == manifest_path
                    || packages.iter().any(|(name, version)| {
                        package.manifest_path.as_std_path()
                            == workspace.join(format!("{name}-{version}/Cargo.toml"))
                    })
            }),
            "consumer has a foreign local package",
        )?;
        let mut command = cargo(toolchain);
        command
            .args(["run", "--locked", "--offline", "--manifest-path"])
            .arg(&manifest_path)
            .arg("--target-dir")
            .arg(workspace.join("consumer-build"));
        check(&mut command, workspace, "consumer-native-0")?;
        let mut command = cargo(toolchain);
        command
            .args([
                "check",
                "--locked",
                "--offline",
                "--target",
                "wasm32-unknown-unknown",
                "--manifest-path",
            ])
            .arg(&manifest_path)
            .arg("--target-dir")
            .arg(workspace.join("consumer-build"));
        check(&mut command, workspace, "consumer-wasm")?;
        require(
            fs::read(&manifest_path)? == manifest.as_bytes()
                && fs::read(consumer.join("Cargo.lock"))? == lock
                && fs::read(consumer.join("src/main.rs"))? == source,
            "consumer inputs changed during qualification",
        )
    }
}

fn consumer_manifest(
    workspace: &Path,
    packages: &[(String, String)],
    msrv: &str,
) -> Result<String> {
    let mut manifest = format!(
        "[workspace]\n[package]\nname = \"storage-package-consumer\"\nversion = \"0.0.0\"\nedition = \"2024\"\nrust-version = {msrv:?}\n[dependencies]\n"
    );
    for (name, version) in packages {
        let path = workspace.join(format!("{name}-{version}"));
        writeln!(
            manifest,
            "{name} = {{ path = {}{} }}",
            serde_json::to_string(&path)?,
            if name == "ic-storage" {
                ", features = [\"test-support\"]"
            } else {
                ""
            }
        )?;
    }
    for (package, dependency) in [
        ("ic-metrics-storage", "ic-metrics"),
        ("ic-logging-storage", "ic-logging"),
    ] {
        let (_, version) = packages
            .iter()
            .find(|(name, _)| name == package)
            .ok_or("missing adapter archive")?;
        let text = fs::read_to_string(workspace.join(format!("{package}-{version}/Cargo.toml")))?;
        let requirement = toml_string(&text, &format!("dependencies.{dependency}"), "version")?;
        writeln!(manifest, "{dependency} = {requirement:?}")?;
    }
    manifest.push_str("[patch.crates-io]\n");
    for (name, version) in packages {
        writeln!(
            manifest,
            "{name} = {{ path = {} }}",
            serde_json::to_string(&workspace.join(format!("{name}-{version}")))?
        )?;
    }
    Ok(manifest)
}

fn cargo(toolchain: &str) -> Command {
    let mut command = Command::new("cargo");
    command.arg(format!("+{toolchain}"));
    command
}

fn check(command: &mut Command, evidence: &Path, stage: &str) -> Result<Vec<u8>> {
    let output = capture_package_command(
        command,
        stage,
        &evidence.join(format!("{stage}.stdout")),
        &evidence.join(format!("{stage}.stderr")),
        PROCESS_LIMITS,
    )?;
    Ok(output.stdout)
}
