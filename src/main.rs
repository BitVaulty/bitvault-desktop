mod app;
mod crypto;
mod icons;
mod wallet;

use app::*;
use leptos::*;

fn main() {
    console_error_panic_hook::set_once();
    logging::log!("csr mode - mounting to body");
    mount_to_body(|| {
        view! {
            <App/>
        }
    })
}
