use sapp_jsutils::JsObject;

extern "C" {
    fn local_storage_get(key: JsObject) -> JsObject;
    fn local_storage_set(key: JsObject, value: JsObject);
}

pub fn get(key: &str) -> Option<String> {
    let value = unsafe { local_storage_get(JsObject::string(key)) };

    if value.is_nil() {
        None
    }
    else {
        let mut buffer = String::new();
        value.to_string(&mut buffer);
        Some(buffer)
    }
}

pub fn set(key: &str, value: &str) {
    let (key_obj, value_obj) = (JsObject::string(key), JsObject::string(value));
    unsafe { local_storage_set(key_obj, value_obj) };
}
