use log::info;

pub enum ScreenManagementCmd {
    None,
    PopScreen,
    PushScreen(Box<dyn ScreenTrait>),
}
pub trait ScreenTrait {
    // fn process_input(&mut self, input: MyInputEvent);
    fn start_scroll(&mut self, pos: (f64, f64)) -> bool {
        info!("YAY start scroll!!!! {:?}", pos);
        true
    }
    fn scroll(&mut self, pos: (f64, f64)) {
        info!("YAY scroll!!!! {:?}", pos);
    }
    fn press(&mut self, pos: (f64, f64)) {
        info!("YAY press!!!! {:?}", pos);
    }
    fn back(&mut self) {
        info!("YAY back button!!!!");
    }
    fn draw(&mut self);
    fn get_screen_management_cmd(&mut self) -> ScreenManagementCmd {
        ScreenManagementCmd::None
    }
}