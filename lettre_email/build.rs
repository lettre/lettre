extern crate skeptic;

use skeptic::*;

fn main() {
    let mut mdbook_files = markdown_files_of_directory("../website/content/creating-messages/");
    // Also add "README.md" to the list of files.
    mdbook_files.push("README.md".into());

    generate_doc_tests(&mdbook_files);
}
