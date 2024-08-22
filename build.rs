use std::process::Command;

fn main() {
    // Specify the path to your shell script
    let script_path = "./generate-schema.sh";

    // Run the shell script
    let status = Command::new("sh")
        .arg(script_path)
        .status()
        .expect("failed to execute generate-schema shell script");

    // Check if the script ran successfully
    if !status.success() {
        panic!(
            "generate-schema shell script failed with status: {:?}",
            status
        );
    }

    // If your script generates files needed by your crate, you should
    // inform Cargo about these files to trigger a rebuild when they change.
    // For example:
    // println!("cargo:rerun-if-changed=src/generated/");
}
