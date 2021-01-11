use sapp_jsutils::JsObject;

extern "C" {
    fn cookie_get(key: JsObject) -> JsObject;
    fn cookie_set(key: JsObject, value: JsObject);
}

pub fn get(key: &str) -> Option<String> {
    let value = unsafe { cookie_get(JsObject::string(key)) };

    if value.is_nil() {
        None
    }
    else {
        let mut buffer = String::new();
        value.to_string(&mut buffer);
        Some(buffer)
    }
}

pub fn set(key: &str, value: &str) { unsafe { cookie_set(JsObject::string(key), JsObject::string(value)) }; }
