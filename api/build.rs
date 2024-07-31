use std::process::Command;

fn main() {
    let script_path = "scripts/generate_openapi_models.sh";
    let status = Command::new("sh")
        .arg(script_path)
        .status()
        .expect("Failed to generate openapi models");
    assert!(status.success(), "Script execution failed");
}
