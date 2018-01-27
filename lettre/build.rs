extern crate skeptic;

use skeptic::*;

fn main() {
    let mdbook_files = markdown_files_of_directory("../website/content/sending-messages/");
    generate_doc_tests(&mdbook_files);
}
