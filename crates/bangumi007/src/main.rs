use std::error::Error;

use rocket::routes;

mod module;
mod ui;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // run http_main in a separate thread
    async_std::task::spawn(module::core::scrobbler_server::http_main());
    // module::core::dev_main::run();
    ui::mainapp::ui_main().unwrap_or(());
    Ok(())
}
