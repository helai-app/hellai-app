use web_sys::{wasm_bindgen::JsCast, Document};

/// A manager for setting and retrieving cookies in a web environment.
pub struct WebClientCookieManager;

impl WebClientCookieManager {
    /// Sets a cookie with the specified key and value, and optional attributes.
    ///
    /// # Arguments
    ///
    /// * `key` - The name of the cookie.
    /// * `value` - The value of the cookie.
    /// * `secure` - A boolean that indicates if the cookie should be set with the `Secure` attribute, making it accessible only over HTTPS.
    /// * `http_only` - A boolean that indicates if the cookie should be set with the `HttpOnly` attribute, preventing JavaScript access.
    /// * `max_age` - An optional value (in seconds) for the `Max-Age` attribute to set an expiration time for the cookie.
    ///
    /// # Panics
    ///
    /// The function will panic if it cannot access the `Document` or set the cookie.
    pub fn set_cookie(key: &str, value: &str, secure: bool, http_only: bool, max_age: Option<i64>) {
        let document = load_document();
        let html_document = document
            .dyn_into::<web_sys::HtmlDocument>()
            .expect("should get html document");

        // Start building the cookie string with the key-value pair
        let mut cookie = format!("{}={};", key, value);

        // Add the Secure attribute if required (for HTTPS only)
        if secure {
            cookie.push_str(" Secure;");
        }

        // Add the HttpOnly attribute if required (prevents access via JavaScript)
        if http_only {
            cookie.push_str(" HttpOnly;");
        }

        // Optionally set the max-age if provided (sets cookie expiration in seconds)
        if let Some(age) = max_age {
            cookie.push_str(&format!(" Max-Age={};", age));
        }

        // Set the cookie in the HTML document
        html_document
            .set_cookie(&cookie)
            .expect("should set cookie");
    }

    /// Retrieves the value of a cookie by its key.
    ///
    /// # Arguments
    ///
    /// * `key` - The name of the cookie to retrieve.
    ///
    /// # Returns
    ///
    /// * `Option<String>` - Returns the value of the cookie if found, or `None` if not found.
    ///
    /// # Panics
    ///
    /// The function will panic if it cannot access the `Document` or read the cookie string.
    pub fn get_cookie(key: &str) -> Option<String> {
        let document = load_document();
        let html_document = document
            .dyn_into::<web_sys::HtmlDocument>()
            .expect("should get html document");

        // Get the complete cookie string from the document
        let cookies = html_document.cookie().expect("should get cookie string");

        // Split the cookie string into individual cookies and search for the key
        for cookie in cookies.split(';') {
            let cookie = cookie.trim(); // Remove leading/trailing whitespace from each cookie
                                        // Split each cookie into a key-value pair
            if let Some((cookie_key, cookie_value)) = cookie.split_once('=') {
                // Return the value if the key matches
                if cookie_key == key {
                    return Some(cookie_value.to_string());
                }
            }
        }

        None // Return None if the cookie with the specified key is not found
    }
}

/// Loads the current web document.
///
/// # Returns
///
/// * `Document` - The `Document` object representing the web page.
///
/// # Panics
///
/// The function will panic if it cannot access the `Window` or `Document` objects.
fn load_document() -> Document {
    web_sys::window()
        .expect("should have access to the Window") // Ensure we have access to the window object
        .document()
        .expect("should have access to the Document") // Ensure we have access to the document object
}
