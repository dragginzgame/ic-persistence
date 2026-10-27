//! Registry observations and upload acknowledgements retain publication state.
use super::*;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum RegistryFault {
    RegistryError,
    RegistryForbidden,
    RegistryYanked,
    StorageUpload,
    UploadResponseLost,
    Checksum,
}

#[derive(Default, Deserialize, Serialize)]
pub(super) struct RegistryEffects {
    pub published: BTreeMap<String, String>,
    pub fail: Option<RegistryFault>,
}

impl RegistryEffects {
    pub(super) fn observe(&self, root: &Path, args: &[&str]) -> Result<String> {
        if self.fail.as_ref() == Some(&RegistryFault::RegistryForbidden) {
            return Ok("forbidden\n403".to_owned());
        }
        if self.fail.as_ref() == Some(&RegistryFault::RegistryError) {
            return Ok("unavailable\n503".to_owned());
        }
        let url = args.last().unwrap();
        let name = url.rsplit('/').nth(1).unwrap();
        let Some(digest) = self.published.get(name) else {
            return Ok("missing\n404".to_owned());
        };
        Ok(format!(
            "{}\n200",
            serde_json::json!({"version": {
                "crate": name, "num": BuildEffects::version(root)?, "checksum": digest,
                "yanked": self.fail.as_ref() == Some(&RegistryFault::RegistryYanked)
            }})
        ))
    }
    pub(super) fn publish(
        &mut self,
        root: &Path,
        args: &[&str],
        git: &mut GitState,
    ) -> Result<String> {
        let target = root.join("configured output");

        let name = args[args.iter().position(|arg| *arg == "--package").unwrap() + 1];
        if !args.contains(&"--dry-run") {
            if name == "ic-storage" && self.fail.as_ref() == Some(&RegistryFault::StorageUpload) {
                return Err("Storage upload refused".into());
            }
            let archive = target
                .join("package")
                .join(format!("{name}-{}.crate", BuildEffects::version(root)?));
            let digest = Repository::<Substitute>::sha256(&archive)?;
            self.published.insert(name.to_owned(), digest);
            if self.fail.as_ref() == Some(&RegistryFault::UploadResponseLost) {
                return Err("response lost after accepted upload".into());
            }
        }
        git.after_publish();
        Ok(String::new())
    }
}
