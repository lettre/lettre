extern crate glob;

use self::glob::glob;
use std::env;
use std::env::consts::EXE_EXTENSION;
use std::path::Path;
use std::process::Command;

#[test]
fn book_test() {
    let mut book_path = env::current_dir().unwrap();
    book_path.push(
        Path::new(file!())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("../website/content/creating-messages"),
    ); // For some reasons, calling .parent() once more gives us None...

    for md in glob(&format!("{}/*.md", book_path.to_str().unwrap())).unwrap() {
        skeptic_test(&md.unwrap());
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

    let result = cmd.spawn()
        .expect("Failed to spawn process")
        .wait()
        .expect("Failed to run process");

    assert!(
        result.success(),
        format!("Failed to run rustdoc tests on {:?}!", path)
    );
}
