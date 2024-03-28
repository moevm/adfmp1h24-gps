use log::info;

pub enum ScreenManagementCmd {
    None,
    PushScreen(Box<dyn ScreenTrait>),
    PopScreen
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
    fn press(&mut self, pos: (f64, f64)) -> ScreenManagementCmd {
        info!("YAY press!!!! {:?}", pos);

        ScreenManagementCmd::None
    }
    fn back(&mut self) -> ScreenManagementCmd {
        info!("YAY back button!!!!");
        ScreenManagementCmd::PopScreen
    }
    fn draw(&mut self);

    fn is_expanded(&self) -> bool {
        false
    }
}