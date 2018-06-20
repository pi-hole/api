use std::process::Command;

fn main() {
    // Read Git data and expose it to the API at compile time
    let tag_raw = Command::new("git")
        .args(&["describe", "--tags", "--abbrev=0", "--exact-match"])
        .output()
        .map(|output| output.stdout)
        .unwrap_or_default();
    let tag = String::from_utf8(tag_raw).unwrap();

    let branch_raw = Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map(|output| output.stdout)
        .unwrap_or_default();
    let branch = String::from_utf8(branch_raw).unwrap();

    let hash_raw = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .map(|output| output.stdout)
        .unwrap_or_default();
    let hash = String::from_utf8(hash_raw).unwrap();

    // This lets us use the `env!()` macro to read these variables at compile time
    println!("cargo:rustc-env=GIT_TAG={}", tag.trim());
    println!("cargo:rustc-env=GIT_BRANCH={}", branch.trim());
    println!("cargo:rustc-env=GIT_HASH={}", hash.trim());
}