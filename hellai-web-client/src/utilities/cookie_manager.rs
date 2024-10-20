use web_sys::{wasm_bindgen::JsCast, Document};

pub struct WebClientCookieManager {}

impl WebClientCookieManager {
    pub fn set_cookie(key: &str, value: &str, secure: bool, http_only: bool, max_age: Option<i64>) {
        let document = load_document();
        let html_document = document
            .dyn_into::<web_sys::HtmlDocument>()
            .expect("should get html document");

        // Start building the cookie string
        let mut cookie = format!("{}={};", key, value);

        // Add the Secure attribute if required
        if secure {
            cookie.push_str(" Secure;");
        }

        // Add HttpOnly if required
        if http_only {
            cookie.push_str(" HttpOnly;");
        }

        // Optionally set max-age if provided
        if let Some(age) = max_age {
            cookie.push_str(&format!(" Max-Age={};", age));
        }

        // Set the cookie
        html_document
            .set_cookie(&cookie)
            .expect("should set cookie");
    }

    pub fn get_cookie(key: &str) -> Option<String> {
        let document = load_document();
        let html_document = document
            .dyn_into::<web_sys::HtmlDocument>()
            .expect("should get html document");

        // Get the cookie string from the document
        let cookies = html_document.cookie().expect("should get cookie string");

        // Split the cookie string into individual cookies
        for cookie in cookies.split(';') {
            let cookie = cookie.trim(); // Remove any leading/trailing whitespace
            if let Some((cookie_key, cookie_value)) = cookie.split_once('=') {
                if cookie_key == key {
                    return Some(cookie_value.to_string()); // Return the value as a String
                }
            }
        }

        None // Return None if the key is not found
    }
}

fn load_document() -> Document {
    web_sys::window()
        .expect("should have access to the Window")
        .document()
        .expect("should have access to the Document")
}
