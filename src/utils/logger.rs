use log4rs;

pub fn init_logging() {
    log4rs::init_file("data/config/log4rs.yaml", Default::default()).unwrap();
}