use mythos_core::base::input::{Key, KeyboardEvent};
use mythos_core::base::logger::Logger;

pub enum Direction {
  Up,
  Down,
  Left,
  Right,
}

pub struct Game {
  direction: Direction,
  logger: Box<dyn Logger>,
}

impl Game {
  pub fn new(logger: Box<dyn Logger>) -> Self {
    Self { logger, direction: Direction::Down }
  }

  pub fn on_key(&mut self, event: KeyboardEvent) {

  }

  pub fn update(&mut self, timestamp: f64) {

  }

  pub fn render(&self, timestamp: f64) -> String {
    format!("snek, tick {}", timestamp).to_string()
  }

  pub async fn install_deps(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
  }
}
