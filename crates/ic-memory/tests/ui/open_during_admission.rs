use ic_memory::BootstrapAdmission;

fn prepare(admission: &BootstrapAdmission<'_>) {
    admission.open_memory(&ic_memory::StableKey::parse("app.journal.v1").unwrap());
}

fn main() {}
