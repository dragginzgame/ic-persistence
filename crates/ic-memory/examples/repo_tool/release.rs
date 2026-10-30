//! Release responsibilities for the repository adapter.
use super::*;

impl<E: Execute> Repository<E> {
    pub(super) fn local_tag(&self, version: &str) -> Result<Option<String>> {
        let refs = self.git(&[
            "for-each-ref",
            "--format=%(objectname) %(refname)",
            "refs/tags",
        ])?;
        let name = format!("refs/tags/v{version}");
        Ok(refs
            .lines()
            .filter_map(|line| line.split_once(' '))
            .find(|(_, reference)| *reference == name)
            .map(|(hash, _)| hash.to_owned()))
    }

    pub(super) fn remote_ready(&self, selection: &ReleaseSelection) -> Result<()> {
        require(
            self.git(&["symbolic-ref", "--quiet", "--short", "HEAD"])? == selection.branch,
            "selected release branch is not checked out",
        )?;
        let remote_ref = format!("refs/remotes/{}/{}", selection.remote, selection.branch);
        self.git(&[
            "fetch",
            "--quiet",
            "--no-tags",
            &selection.remote,
            &format!("+refs/heads/{}:{remote_ref}", selection.branch),
        ])?;
        self.git(&["merge-base", "--is-ancestor", &remote_ref, "HEAD"])?;
        Ok(())
    }

    pub(super) fn commit_check(&self, selection: &ReleaseSelection) -> Result<()> {
        let validation = self.preparation_validation(selection)?;
        let surfaces = self.surfaces(selection)?;
        self.qualify_prepared(validation, &surfaces)?;
        self.check_index(selection, &surfaces)?;
        require(
            self.git(&["diff", "--name-only"])?.is_empty(),
            "release metadata has unstaged changes",
        )?;
        require(
            self.git(&["diff", "--cached", "--name-only"])?
                == self.git(&["diff", "--name-only", &selection.source])?,
            "stage exactly the release metadata",
        )
    }

    pub(super) fn inspect_release_commit(&self, commit: &str) -> Result<InspectedRelease> {
        self.clean()?;
        let version = self.version(Some(commit))?;
        let prepared: PackageEvidence =
            serde_json::from_slice(&fs::read(self.receipt_path(&version, Prepared)?)?)?;
        let selection = prepared.validation.selection;
        require(
            selection.version == version,
            "release metadata differs from saved intent",
        )?;
        require(
            self.git(&["log", "-1", "--format=%P", commit])? == selection.source,
            "release parent differs from validated source",
        )?;
        require(
            self.git(&["log", "-1", "--format=%s", commit])? == format!("Release {version}"),
            "selected commit is not the release commit",
        )?;
        self.git(&["merge-base", "--is-ancestor", commit, "HEAD"])?;
        self.check_surfaces(
            &selection,
            &self.surfaces(&selection)?,
            Some(commit),
            SurfaceState::Candidate,
        )?;
        Ok(InspectedRelease {
            commit: commit.to_owned(),
            selection,
        })
    }

    pub(super) fn release_commit(&self) -> Result<InspectedRelease> {
        let release = self.inspect_release_commit(&self.git(&["rev-parse", "HEAD"])?)?;
        self.verify_evidence(&release.selection, &release.commit, Final, Working)?;
        Ok(release)
    }

    pub(super) fn qualify_release(&self) -> Result<()> {
        self.qualify_release_at(&self.git(&["rev-parse", "HEAD"])?)
    }

    pub(super) fn qualify_release_at(&self, commit: &str) -> Result<()> {
        let release = self.inspect_release_commit(commit)?;
        let selection = &release.selection;
        let version = &selection.version;
        let head = &release.commit;
        let mut evidence =
            self.verify_evidence(selection, &selection.source, Prepared, Retained)?;
        let current = self.git(&["rev-parse", "HEAD"])? == commit;
        if self.receipt_path(version, Final)?.exists() {
            // Recovery verifies the retained archive for the selected commit.
            // Cargo's working archive may belong to the newer source at HEAD.
            self.verify_evidence(
                selection,
                head,
                Final,
                if current { Working } else { Retained },
            )?;
            return Ok(());
        }
        require(
            current,
            "selected older release has no final qualification receipt; qualify that exact commit before recovery",
        )?;
        let command = self.final_package_command()?;
        self.run_command(&command)?;
        require(
            self.git(&["rev-parse", "HEAD"])? == commit
                && self.inspect_release_commit(commit)? == release,
            "release changed during final qualification",
        )?;
        self.verify_inputs(&evidence.validation, LockState::ReleaseProjection)?;
        require(
            Self::sha256(&self.root.join("Cargo.lock"))? == evidence.lock_sha256,
            "final packaging changed selected dependencies",
        )?;
        evidence.commands.push(command);
        let evidence = PackageEvidence::record(
            self,
            evidence.validation,
            head.clone(),
            evidence.lock_sha256,
            evidence.commands,
        )?;
        self.write_evidence(&evidence, Final)
    }

    pub(super) fn selected_committed_check(
        &self,
        selection: &ReleaseSelection,
        commit: &str,
    ) -> Result<()> {
        require(
            self.inspect_release_commit(commit)?.selection == *selection,
            "commit identifies another release intent",
        )?;
        self.qualify_release_at(commit)
    }

    pub(super) fn tagged_check(
        &self,
        selection: &ReleaseSelection,
        commit: &str,
    ) -> Result<InspectedRelease> {
        let release = self.inspect_release_commit(commit)?;
        require(
            release.selection == *selection,
            "tagged release identifies another intent",
        )?;
        self.verify_evidence(
            selection,
            commit,
            Final,
            if self.git(&["rev-parse", "HEAD"])? == commit {
                Working
            } else {
                Retained
            },
        )?;
        self.check_tag(&release.selection.version, &release.commit)?;
        Ok(release)
    }

    pub(super) fn push_check(&self, selection: &ReleaseSelection, commit: &str) -> Result<()> {
        let release = self.tagged_check(selection, commit)?;
        self.remote_ready(selection)?;
        require(
            self.tagged_check(selection, commit)? == release,
            "release changed during remote inspection",
        )?;
        Ok(())
    }

    pub(super) fn check_tag(&self, version: &str, head: &str) -> Result<String> {
        let tag = self.local_tag(version)?.ok_or("release tag is missing")?;
        self.output("bash", &[TAG_CHECKER, head, version])?;
        require(
            self.local_tag(version)?.as_deref() == Some(tag.as_str()),
            "release tag changed during admission",
        )?;
        Ok(tag)
    }
}
