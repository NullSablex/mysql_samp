use samp::prelude::*;

use crate::connection::ConnectionManager;
use crate::logger::Logger;
use crate::options::OptionsManager;

pub struct MysqlPlugin {
    pub connections: ConnectionManager,
    pub options: OptionsManager,
}

impl MysqlPlugin {
    pub fn new() -> Self {
        Logger::init();

        Self {
            connections: ConnectionManager::new(),
            options: OptionsManager::new(),
        }
    }
}

impl SampPlugin for MysqlPlugin {
    fn on_load(&mut self) {}

    fn on_unload(&mut self) {
        Logger::info("Plugin unloaded.");
    }
}
