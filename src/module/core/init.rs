use crate::module;

pub fn run_init() -> Result<(), Box<dyn std::error::Error>> {
    module::logger::init();
    module::database::init_database()?;
    Ok(())
}