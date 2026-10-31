//! Consumer selection over the canonical shared release-group policy.
use super::*;

pub(super) const GROUP_POLICY: &str = "ci/cargo-release-groups.json";

///
/// ReleaseGroup
///
/// Consumer metadata selected from the canonical Shared group policy.
///

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReleaseGroup {
    pub name: String,
    pub primary: String,
    pub version: String,
    pub members: Vec<String>,
    pub changelog: String,
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub metadata_files: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReleaseGroups {
    schema: u32,
    groups: Vec<ReleaseGroup>,
}

impl<E: Execute> Repository<E> {
    pub(super) fn release_groups(&self, revision: Option<&str>) -> Result<Vec<ReleaseGroup>> {
        let policy: ReleaseGroups = serde_json::from_str(&self.text(GROUP_POLICY, revision)?)?;
        require(policy.schema == 1, "unsupported release-group policy")?;
        for group in &policy.groups {
            let identifier = |s: &str| {
                !s.is_empty()
                    && s.bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
            };
            let relative = |s: &str| {
                !s.is_empty()
                    && !s.bytes().any(|b| b.is_ascii_control())
                    && Path::new(s)
                        .components()
                        .all(|c| matches!(c, std::path::Component::Normal(_)))
            };
            require(
                identifier(&group.name)
                    && identifier(&group.primary)
                    && group.members.iter().all(|name| identifier(name))
                    && group.members.contains(&group.primary)
                    && group.version == "package"
                    && relative(&group.changelog)
                    && group.metadata_files.iter().all(|path| relative(path)),
                "invalid consumer release group",
            )?;
        }
        Ok(policy.groups)
    }

    pub(super) fn release_group(&self, revision: Option<&str>) -> Result<ReleaseGroup> {
        require(!self.group.is_empty(), "select RELEASE_GROUP explicitly")?;
        self.release_groups(revision)?
            .into_iter()
            .find(|group| group.name == self.group)
            .ok_or_else(|| "unknown selected release group".into())
    }

    pub(super) fn release_files(&self, revision: Option<&str>) -> Result<Vec<String>> {
        let group = self.release_group(revision)?;
        let mut paths = vec![
            "Cargo.toml".to_owned(),
            "Cargo.lock".to_owned(),
            group.changelog,
        ];
        paths.extend(group.members.iter().map(|name| package_manifest(name)));
        paths.extend(group.metadata_files);
        Ok(paths)
    }

    pub(super) fn selected_packages(&self) -> Result<Vec<String>> {
        Ok(self.release_group(None)?.members)
    }

    pub(super) fn check_lock_update(
        &self,
        before: &[u8],
        after: &str,
        previous: &str,
        version: &str,
    ) -> Result<()> {
        let group = self.release_group(None)?;
        verify_workspace_release_projection(
            std::str::from_utf8(before)?,
            after,
            &group
                .members
                .iter()
                .map(|package| WorkspaceVersionProjection {
                    package,
                    previous,
                    next: version,
                })
                .collect::<Vec<_>>(),
        )?;
        Ok(())
    }

    // The consumer catalog uses bare compatible stable requirements. Preserve a
    // compatible minimum; a breaking supplier release must advance its consumers'
    // shared requirement, without advancing their independently owned versions.
    pub(super) fn compatible_requirement(&self, requirement: &str, version: &str) -> Result<bool> {
        let required = self.version_parts(requirement)?;
        let next = self.version_parts(version)?;
        Ok(next >= required
            && if required[0] != 0 {
                required[0] == next[0]
            } else if required[1] != 0 {
                required[..2] == next[..2]
            } else {
                required == next
            })
    }
}
