use std::io::Read;

mod bindings {
    wit_bindgen::generate!({ generate_all });

    use super::UsbFileReader;
    export!(UsbFileReader);
}

struct UsbFileReader;

impl bindings::exports::idlab::file::reader::Guest for UsbFileReader {
    fn read(path: String) -> Result<String, String> {
        let fs = usb_storage::mount().map_err(|e| format!("Failed to mount USB: {e}"))?;
        let root = fs.root_dir();

        let mut file = root
            .open_file(&path)
            .map_err(|e| format!("Failed to open '{path}': {e}"))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read '{path}': {e}"))?;

        Ok(contents)
    }
}
