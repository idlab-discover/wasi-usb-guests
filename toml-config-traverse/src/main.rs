mod bindings {
    wit_bindgen::generate!({
        generate_all,
    });
}

use bindings::idlab::config::provider;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    traverse_keys("", 0)?;
    Ok(())
}

fn traverse_keys(prefix: &str, depth: usize) -> Result<(), Box<dyn std::error::Error>> {
    let keys = provider::list_keys(prefix)?;

    for key in &keys {
        let full_key = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{}.{}", prefix, key)
        };

        let indent = "  ".repeat(depth);

        let mut found = false;

        // Traverse if this is a table
        if let Ok(nested_keys) = provider::list_keys(&full_key)
            && !nested_keys.is_empty()
        {
            println!("{}[{}]", indent, key);
            traverse_keys(&full_key, depth + 1)?;
            found = true;
        }

        // If not a table, it's a value
        if !found {
            if let Ok(Some(value)) = provider::get_string(&full_key) {
                println!("{}{} = \"{}\"", indent, key, value);
                found = true;
            } else if let Ok(Some(value)) = provider::get_integer(&full_key) {
                println!("{}{} = {}", indent, key, value);
                found = true;
            } else if let Ok(Some(value)) = provider::get_float(&full_key) {
                println!("{}{} = {}", indent, key, value);
                found = true;
            } else if let Ok(Some(value)) = provider::get_bool(&full_key) {
                println!("{}{} = {}", indent, key, value);
                found = true;
            } else if let Ok(Some(list)) = provider::get_string_list(&full_key) {
                println!("{}{} = {:?}", indent, key, list);
                found = true;
            }

            if !found {
                println!("{}{} = <unknown>", indent, key);
            }
        }
    }

    Ok(())
}
