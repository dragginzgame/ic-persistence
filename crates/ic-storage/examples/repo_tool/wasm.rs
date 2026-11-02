//! Wasm responsibilities for the repository adapter.
use super::*;

impl<E: Execute> Repository<E> {
    pub(super) fn wasm_size(&self) -> Result<()> {
        let mut command = vec![
            "cargo".to_owned(),
            format!("+{}", self.toolchain()?),
            "build".to_owned(),
            "--locked".to_owned(),
            "--offline".to_owned(),
            "--profile".to_owned(),
            "wasm-size".to_owned(),
            "--target".to_owned(),
            "wasm32-unknown-unknown".to_owned(),
        ];
        for (probe, _) in PROBES {
            command.extend([
                "--example".to_owned(),
                format!("wasm-{}-size-probe", probe.replace('_', "-")),
            ]);
        }
        self.run_command(&command)?;
        check_wasm_artifacts(&self.target()?)
    }
}

pub(super) const PROBES: [(&str, u64); 4] = [
    ("core", 260_000),
    ("diagnostics", 315_000),
    ("admission", 264_000),
    ("runtime_integration", 270_000),
];

pub(super) fn check_wasm_artifacts(target: &Path) -> Result<()> {
    let artifacts = target.join("wasm32-unknown-unknown/wasm-size/examples");
    for (probe, budget) in PROBES {
        let path = artifacts.join(format!("wasm_{probe}_size_probe.wasm"));
        let metadata = fs::metadata(&path)?;
        require(metadata.is_file(), "Wasm artifact is not a regular file")?;
        let bytes = metadata.len();
        println!("{probe} raw Wasm: {bytes} bytes (budget: {budget} bytes)");
        require(
            bytes <= budget,
            &format!("{} exceeds its raw Wasm budget", path.display()),
        )?;
    }
    Ok(())
}
