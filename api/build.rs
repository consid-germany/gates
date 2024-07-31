use std::process::Command;

fn main() {
    // Specify the path to your script relative to the project root
    let script_path = "scripts/generate_openapi_models.sh";

    // Run the script
    let status = Command::new("sh")
        .arg(script_path)
        .status()
        .expect("Failed to execute script");

    if !status.success() {
        panic!("Script execution failed");
    }
}