use std::env::{self, consts::EXE_EXTENSION};
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

#[test]
fn book_test() {
    // README needs to be compatible with latest release
    //skeptic_test(Path::new("README.md"));

    for entry in WalkDir::new("website").into_iter().filter(|e| {
        e.as_ref()
            .unwrap()
            .path()
            .extension()
            .map(|ex| ex == "md")
            .unwrap_or(false)
    }) {
        skeptic_test(entry.unwrap().path());
    }
}

fn skeptic_test(path: &Path) {
    let rustdoc = Path::new("rustdoc").with_extension(EXE_EXTENSION);
    let exe = env::current_exe().unwrap();
    let depdir = exe.parent().unwrap();

    let mut cmd = Command::new(rustdoc);
    cmd.args(&["--verbose", "--test"])
        .arg("-L")
        .arg(&depdir)
        .arg(path);

    let result = cmd
        .spawn()
        .expect("Failed to spawn process")
        .wait()
        .expect("Failed to run process");
    assert!(
        result.success(),
        format!("Failed to run rustdoc tests on {:?}", path)
    );
}
