mod bindings {
    wit_bindgen::generate!({
        generate_all,
    });
}

pub use bindings::wasi::usb::device;
