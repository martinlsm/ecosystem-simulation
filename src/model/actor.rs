pub trait Actor {
    fn update(&mut self, dt: f32);
    fn do_action(&mut self, action: Action) -> Result<(), &'static str>;
    fn get_pos(&self) -> (f32, f32);
}

pub enum Action {
    Forward,
    Stop,
    Turn(f32),
}
