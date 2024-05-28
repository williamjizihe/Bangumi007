mod module;
mod ui;

fn main() {
    // module::core::dev_main::run();
    ui::helloworld::ui_main().unwrap_or(());
}
