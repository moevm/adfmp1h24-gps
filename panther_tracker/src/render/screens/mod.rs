pub mod main;

use log::info;

pub enum ScreenManagementCmd {
    None,
    PushScreen(Box<dyn ScreenTrait>),
    PopScreen
}
pub trait ScreenTrait {
    fn start_scroll(&mut self, pos: (f64, f64)) -> bool {
        true
    }
    fn scroll(&mut self, pos: (f64, f64)) {
        // info!("YAY scroll!!!! {:?}", pos);
    }
    fn press(&mut self, pos: (f64, f64)) -> ScreenManagementCmd {
        info!("YAY press!!!! {:?}", pos);

        ScreenManagementCmd::None
    }
    fn back(&mut self) -> ScreenManagementCmd {
        ScreenManagementCmd::None
    }
    fn draw(&mut self);

    fn is_expanded(&self) -> bool {
        false
    }
}