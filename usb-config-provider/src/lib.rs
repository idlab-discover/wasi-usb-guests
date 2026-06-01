use std::io::Read;
use std::sync::OnceLock;

mod bindings {
    wit_bindgen::generate!({ generate_all });

    use super::UsbConfigProvider;
    export!(UsbConfigProvider);
}

const FILE_PATH: &str = "policy.toml";

static CONFIG: OnceLock<Result<toml::Value, String>> = OnceLock::new();

fn load_config() -> Result<toml::Value, String> {
    let fs = usb_storage::mount().map_err(|e| format!("Failed to mount USB: {e}"))?;
    let root = fs.root_dir();

    let mut file = root
        .open_file(FILE_PATH)
        .map_err(|e| format!("Failed to open '{FILE_PATH}': {e}"))?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Failed to read '{FILE_PATH}': {e}"))?;

    toml::from_str(&contents).map_err(|e| format!("Failed to parse '{FILE_PATH}': {e}"))
}

fn config() -> Result<&'static toml::Value, String> {
    CONFIG
        .get_or_init(load_config)
        .as_ref()
        .map_err(|e| e.clone())
}

fn resolve<'a>(root: &'a toml::Value, key: &str) -> Option<&'a toml::Value> {
    let mut current = root;
    for segment in key.split('.') {
        current = current.as_table()?.get(segment)?;
    }
    Some(current)
}

fn keys_at(root: &toml::Value, prefix: &str) -> Option<Vec<String>> {
    let table = if prefix.is_empty() {
        root.as_table()?
    } else {
        resolve(root, prefix)?.as_table()?
    };
    Some(table.keys().cloned().collect())
}

struct UsbConfigProvider;

impl bindings::exports::idlab::config::provider::Guest for UsbConfigProvider {
    fn get_string(key: String) -> Result<Option<String>, String> {
        let cfg = config()?;
        Ok(resolve(cfg, &key).and_then(|v| v.as_str().map(String::from)))
    }

    fn get_integer(key: String) -> Result<Option<i64>, String> {
        let cfg = config()?;
        Ok(resolve(cfg, &key).and_then(|v| v.as_integer()))
    }

    fn get_float(key: String) -> Result<Option<f64>, String> {
        let cfg = config()?;
        Ok(resolve(cfg, &key).and_then(|v| v.as_float()))
    }

    fn get_bool(key: String) -> Result<Option<bool>, String> {
        let cfg = config()?;
        Ok(resolve(cfg, &key).and_then(|v| v.as_bool()))
    }

    fn get_string_list(key: String) -> Result<Option<Vec<String>>, String> {
        let cfg = config()?;
        Ok(resolve(cfg, &key).and_then(|v| {
            v.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(String::from))
                    .collect()
            })
        }))
    }

    fn list_keys(prefix: String) -> Result<Vec<String>, String> {
        let cfg = config()?;
        Ok(keys_at(cfg, &prefix).unwrap_or_default())
    }
}
