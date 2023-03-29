use std::{env, process::Command};

use anyhow::{bail, Result};

fn main() -> Result<()> {
    let cwd = env::var("CARGO_MANIFEST_DIR")?;

    // Build the rika firenet mock image in the repository
    let output = Command::new("docker")
        .arg("build")
        .arg("--file")
        .arg(&format!("{cwd}/../mock/Dockerfile"))
        .arg("--tag")
        .arg("rika-firenet-api-mock:latest")
        .arg(&format!("{cwd}/../mock"))
        .output()?;
    if !output.status.success() {
        eprintln!("stderr: {}", String::from_utf8(output.stderr)?);
        bail!("unable to build rika-firenet-api-mock:latest:latest");
    }
    eprintln!("Built rika-firenet-api-mock:latest");

    // trigger recompilation when dockerfiles are modified
    println!("cargo:rerun-if-changed=../mock");

    Ok(())
}
