use std::process::Command;

fn main() {
    // Read Git data and expose it to the API at compile time
    let tag_raw = Command::new("git")
        .args(&["describe", "--tags", "--abbrev=0", "--exact-match"])
        .output()
        .expect("Failed to get git tag");
    let tag = String::from_utf8(tag_raw.stdout).unwrap();

    let branch_raw = Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("Failed to get git branch");
    let branch = String::from_utf8(branch_raw.stdout).unwrap();

    let hash_raw = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .expect("Failed to get commit hash");
    let hash = String::from_utf8(hash_raw.stdout).unwrap();

    // This lets us use the `env!()` macro to read these variables at compile time
    println!("cargo:rustc-env=GIT_TAG={}", tag);
    println!("cargo:rustc-env=GIT_BRANCH={}", branch);
    println!("cargo:rustc-env=GIT_HASH={}", hash.get(0..7).unwrap()); // Get a short hash
}