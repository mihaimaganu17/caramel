use std::path::Path;
use std::error::Error;
use std::process::Command;

/// Check if a command is working and returning the expected results.
fn check_install(command: &str, args: &[&str],
                expected: &[&str]) -> Option<()> {
    // Invoke the command
    let result = Command::new(command).args(args).output().ok()?;

    // Check if the command was successful
    if !result.status.success() { return None; }

    // Convert the stdout bytes to a string
    let stdout = std::str::from_utf8(&result.stdout).ok()?;

    // Make sure `stdout` contains everything we expected
    if expected.iter().all(|x| stdout.contains(x)) {
        Some(())
    } else {
        None
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Check for nasm
    check_install("nasm", &["-v"], &["NASM version"])
        .ok_or("nasm not present in the path")?;

    // Check for rust and needed targets
    check_install("rustup", &["target", "list"],
        &["i586-pc-windows-msvc (installed)",
            "x86_64-pc-windows-msvc (installed)",
        ]).ok_or("rustup not present or i586-pc-windows-msvc or \
                    x86_64-pc-windows-msvc targets not installed")?;

    // Check for ld64.lld
    check_install("lld-link", &["--version"], &["LLD" ]).
        ok_or("ld64.lld not present in the path")?;

    // Check a buld folder, if it does not exist
    std::fs::create_dir_all("build")?;
    std::fs::create_dir_all("build/bootloader")?;

    // Create the boot filename
    let bootfile = Path::new("build").join("caramel.boot");

    // Build the bootloader
    let bootloader_build_dir = Path::new("build").join("bootloader").canonicalize()?;

    if !Command::new("cargo")
            .current_dir("bootloader")
            .args(&["build", "--release", "--target-dir",
                    bootloader_build_dir.to_str().unwrap()])
            .status()?.success() {
        return Err("Failed to build bootloader".into());
    }

    // Build the stage0
    let stage0 = Path::new("bootloader").join("src").join("stage0.asm");
    if !Command::new("nasm")
            .args(&["-f", "bin", "-o", bootfile.to_str().unwrap(), stage0.to_str().unwrap()])
            .status()?.success() {
        return Err("Failed to assemble stage0".into());
    }

    // Deploy the images to the PXE directory
    std::fs::copy(bootfile, "/Users/m3m0ry/fun/netboot/caramel.boot")?;

    Ok(())
}
