use std::process::Command;

fn main() {
    let status = Command::new("sh")
        .arg("scripts/generate_openapi_models.sh")
        .status()
        .expect("Failed to generate openapi models");
    assert!(status.success(), "Script execution failed");
}
