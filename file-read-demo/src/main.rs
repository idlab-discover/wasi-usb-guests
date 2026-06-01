mod bindings {
    wit_bindgen::generate!({
        generate_all,
    });
}

use bindings::idlab::file::reader;

const FILE_PATH: &str = "policy.toml";

fn main() {
    match reader::read(FILE_PATH) {
        Ok(contents) => print!("{contents}"),
        Err(e) => {
            eprintln!("Error reading '{FILE_PATH}': {e}");
            std::process::exit(1);
        }
    }
}
