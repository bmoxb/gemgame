#[cfg(target_arch = "wasm32")]
mod cookies;

const CLIENT_ID_KEY: &str = "client_id";

use shared::Id;

pub fn store_client_id(id: Id) {
    #[cfg(target_arch = "wasm32")]
    cookies::set(CLIENT_ID_KEY, id.encode());

    #[cfg(not(target_arch = "wasm32"))]
    unimplemented!()
}

pub fn retrieve_client_id() -> Option<Id> {
    #[cfg(target_arch = "wasm32")]
    return Id::decode(cookies::get(CLIENT_ID_KEY));

    #[cfg(not(target_arch = "wasm32"))]
    unimplemented!()
}
