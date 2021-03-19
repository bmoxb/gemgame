#[cfg(target_arch = "wasm32")]
mod browser;
#[cfg(not(target_arch = "wasm32"))]
mod desktop;

#[cfg(target_arch = "wasm32")]
const LOCAL_STORAGE_KEY: &str = "clientid";
#[cfg(not(target_arch = "wasm32"))]
const FILE_PATH: &str = "clientid.txt";

use shared::Id;

pub fn store_client_id(id: Id) {
    let encoded = id.encode();

    #[cfg(target_arch = "wasm32")]
    browser::set(KEY, &encoded);

    #[cfg(not(target_arch = "wasm32"))]
    desktop::set(FILE_PATH, &encoded).unwrap(); // TODO: Don't just unwrap!
}

pub fn retrieve_client_id() -> Option<Id> {
    #[cfg(target_arch = "wasm32")]
    return Id::decode(&browser::get(KEY)?);

    #[cfg(not(target_arch = "wasm32"))]
    Id::decode(&desktop::get(FILE_PATH).ok()?)
}
