use super::*;

fn record() -> serde_json::Value {
    serde_json::json!({
        "version": {
            "crate": "ic-storage", "num": "0.13.0", "yanked": false, "checksum": "digest",
            "additional_registry_field": true,
        },
        "additional_response_field": [],
    })
}

fn observe(record: &serde_json::Value) -> Result<bool> {
    registry_response(&format!("{record}\n200"), "ic-storage", "0.13.0", "digest")
}

#[test]
fn registry_identity_requires_matching_unyanked_bytes() {
    assert!(observe(&record()).unwrap());
    for (field, value) in [
        ("crate", serde_json::json!("foreign-package")),
        ("num", serde_json::json!("0.13.1")),
        ("checksum", serde_json::json!("different")),
        ("yanked", serde_json::json!(true)),
    ] {
        let mut record = record();
        record["version"][field] = value;
        assert!(observe(&record).is_err());
    }
}

#[test]
fn registry_facts_must_be_present_and_typed() {
    for field in ["crate", "num", "checksum", "yanked"] {
        let mut missing = record();
        missing["version"].as_object_mut().unwrap().remove(field);
        assert!(observe(&missing).is_err());
        let mut wrong_type = record();
        wrong_type["version"][field] = serde_json::json!([]);
        assert!(observe(&wrong_type).is_err());
    }
    for record in [serde_json::json!({}), serde_json::json!({"version": null})] {
        assert!(observe(&record).is_err());
    }
}

#[test]
fn only_an_explicit_not_found_status_means_absence() {
    assert!(!registry_response("not-json\n404", "ic-storage", "0.13.0", "digest").unwrap());
    for response in ["", "{}", "{}\n403", "{}\n503", "not-json\n200", "{}\n200 "] {
        assert!(registry_response(response, "ic-storage", "0.13.0", "digest").is_err());
    }
}
