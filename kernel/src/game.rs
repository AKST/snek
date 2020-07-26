use mythos_core::base::geometry::*;
use mythos_core::base::input::{Key, KeyboardEvent};
use mythos_core::base::logger::Logger;
use rand::random;
use std::collections::BinaryHeap;
use std::collections::LinkedList;

pub struct Game {
  cherry: Vector2D<i16>,
  freshness: f64,
  dimensions: Vector2D<i16>,
  position: Vector2D<i16>,
  direction: Direction,
  tail: LinkedList<Vector2D<i16>>,
  tail_ghost: LinkedList<Vector2D<i16>>,
  logger: Box<dyn Logger>,
}

pub enum GameEff {
  End,
  Cont,
}

const TICK_RATE: f64 = 4.0 * (1000.0 / 60.0);

impl Game {
  pub fn new(logger: Box<dyn Logger>) -> Self {
    let position = Vector2D::scalar(25);
    let dimensions = Vector2D::scalar(51);
    let mut tail = LinkedList::new();
    let mut tail_ghost = LinkedList::new();

    for i in 0..15 {
      tail.push_back(position - Vector2D::new(0, 1 + i));
    }

    for i in 0..5 {
      tail_ghost.push_back(position - Vector2D::new(0, 16 + i));
    }

    Self {
      cherry: Self::create_cherry(&dimensions, &tail),
      freshness: 0.0,
      dimensions,
      direction: Direction::Down,
      logger,
      position,
      tail,
      tail_ghost,
    }
  }

  pub fn on_key(&mut self, event: KeyboardEvent) {
    if let Some(direction) = Direction::from_keyboard_event(event) {
      self.direction = direction;
    }
  }

  pub fn update(&mut self, timestamp: f64) -> GameEff {
    if timestamp - self.freshness < TICK_RATE {
      return GameEff::Cont;
    }

    if self.position != self.cherry {
      let dead = self.tail.pop_back();
      self.tail_ghost.push_front(dead.unwrap());
    } else {
      self.cherry = Self::create_cherry(&self.dimensions, &self.tail);
      self.tail.append(&mut self.tail_ghost);
    }

    if self.tail_ghost.len() > 5 {
      self.tail_ghost.pop_back();
    }

    self.tail.push_front(self.position);

    self.position = self.position + self.direction.velocity();
    self.position = (self.position + self.dimensions) % self.dimensions;
    self.freshness = timestamp;

    if self.tail.contains(&self.position) {
      GameEff::End
    } else {
      GameEff::Cont
    }
  }

  pub fn render(&self, _timestamp: f64) -> String {
    let mut output = String::from("");
    output.push_str(&format!("pos: {:?}\n\n\n", self.position));
    output.push_str(&horizontal_wall(self.dimensions.x()));
    output.push_str("\n");

    let mut render_queue = self.render_heap();
    let mut render_next = render_queue.pop();

    for y in 0..self.dimensions.y() {
      output.push_str("|");

      for x in 0..self.dimensions.x() {
        let weight = QueuedTile::weight(Vector2D::new(x, y), self.dimensions);
        let maybe_character = render_next.filter(|q| q.1 == weight);

        let character = if let Some(character) = maybe_character {
          render_next = render_queue.pop();
          character.0
        } else {
          Tile::Space
        };

        output.push_str(&character.draw());
      }

      output.push_str("|\n");
    }

    output.push_str(&horizontal_wall(self.dimensions.x()));

    output
  }

  pub async fn install_deps(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
  }

  fn render_heap(&self) -> BinaryHeap<QueuedTile> {
    let mut heap = BinaryHeap::new();
    heap.push(QueuedTile::new_within(Tile::SnakeHead, self.position, self.dimensions));

    for node in self.tail.iter() {
      let tile = Tile::SnakeTail;
      heap.push(QueuedTile::new_within(tile, *node, self.dimensions));
    }

    for node in self.tail_ghost.iter() {
      let tile = Tile::SnakeTailGhost;
      heap.push(QueuedTile::new_within(tile, *node, self.dimensions));
    }

    heap.push(QueuedTile::new_within(Tile::Cherry, self.cherry, self.dimensions));

    heap
  }

  fn create_cherry(
    bounds: &Vector2D<i16>,
    tail: &LinkedList<Vector2D<i16>>,
  ) -> Vector2D<i16> {
    'outer: loop {
      let x = random::<i16>().abs();
      let y = random::<i16>().abs();
      let cherry = Vector2D::new(x, y) % bounds;

      for node in tail.iter() {
        if *node == cherry {
          continue 'outer;
        }
      }

      return cherry;
    }
  }
}

fn horizontal_wall(size: i16) -> String {
  let mut wall = String::from("+");
  let line: String = std::iter::repeat("-").take(size as usize).collect();
  wall.push_str(&line);
  wall.push_str("+");
  wall
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Tile {
  SnakeHead,
  SnakeTail,
  SnakeTailGhost,
  Cherry,
  Space,
}

impl Tile {
  pub fn draw(&self) -> String {
    match self {
      Self::Cherry => "c".to_string(),
      Self::SnakeHead => "x".to_string(),
      Self::SnakeTail => "o".to_string(),
      Self::SnakeTailGhost => ".".to_string(),
      Self::Space => " ".to_string(),
    }
  }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct QueuedTile(pub Tile, pub i16);

impl QueuedTile {
  pub fn new_within(t: Tile, p: Vector2D<i16>, bounds: Vector2D<i16>) -> Self {
    QueuedTile(t, QueuedTile::weight(p, bounds))
  }

  pub fn weight(p: Vector2D<i16>, bounds: Vector2D<i16>) -> i16 {
    let max = bounds.x() * bounds.y();
    max - (p.x() + (p.y() * bounds.x()))
  }
}

impl Ord for QueuedTile {
  fn cmp(&self, other: &QueuedTile) -> std::cmp::Ordering {
    self.1.cmp(&other.1)
  }
}

impl PartialOrd for QueuedTile {
  fn partial_cmp(&self, other: &QueuedTile) -> Option<std::cmp::Ordering> {
    self.1.partial_cmp(&other.1)
  }
}

#[derive(Debug)]
pub enum Direction {
  Up,
  Down,
  Left,
  Right,
}

impl Direction {
  pub fn from_keyboard_event(event: KeyboardEvent) -> Option<Self> {
    let key = match event {
      KeyboardEvent::Down(k) => Some(k),
      KeyboardEvent::Up(_) => None,
    }?;

    match key {
      Key::Char('w') => Some(Self::Up),
      Key::Char('s') => Some(Self::Down),
      Key::Char('a') => Some(Self::Left),
      Key::Char('d') => Some(Self::Right),
      _ => None,
    }
  }

  pub fn velocity(&self) -> Vector2D<i16> {
    match self {
      Self::Up => Vector2D::new(0, -1),
      Self::Down => Vector2D::new(0, 1),
      Self::Left => Vector2D::new(-1, 0),
      Self::Right => Vector2D::new(1, 0),
    }
  }
}
