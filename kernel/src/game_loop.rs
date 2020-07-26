use mythos_core::base::input::{InputEvent, KeyboardEvent};
use mythos_core::base::logger::Logger;
use mythos_core::service::input::InputService;
use mythos_web::bindings::input::{WebInputService};
use mythos_web::bindings::logger::{WebLoggerService, local::LocalConsoleLogger};
use mythos_web::game_loop::*;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlPreElement, Text};

use crate::game::Game;

pub struct DemoGameLoop {
  game: Game,
  input_service: Box<dyn InputService>,
  logger: Box<dyn Logger>,
  text_node: Text,
}

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = console, js_name = log)]
  fn console_log(s: &str);

  #[wasm_bindgen(js_namespace = console, js_name = error)]
  fn console_error(s: &str);
}

impl GameLoop for DemoGameLoop {
  fn create() -> GameLoopResult<Self> {
    Ok(initialise()?)
  }

  fn install<'a>(& 'a mut self) -> AsyncGameLoopResult<'a, ()> {
    Box::pin(self.game.install_deps())
  }

  fn on_animation_frame(&mut self, t: f64) -> GameLoopResult<()> {
    while let Some(event) = self.input_service.poll() {
      self.logger.debug(&format!("event {:?}", event));
      if let InputEvent::Keyboard(e) = event {
        self.game.on_key(e);
      }
    }

    self.text_node.set_data(&self.game.render(t));
    Ok(())
  }

  fn on_init_error(error: &dyn std::error::Error) -> String {
    format!("{}", error)
  }

  fn on_error(&self, error: &dyn std::error::Error) {
    console_error(&Self::on_init_error(error));
  }
}

fn initialise() -> Result<DemoGameLoop, error::DemoError> {
  util::style_page()?;

  let body = util::get_body()?;
  let window = util::get_window()?;
  let pre_node = util::create_element::<HtmlPreElement>("pre")?;
  let text_node = Text::new()?;
  pre_node.append_child(&text_node)?;
  body.append_child(&pre_node)?;

  let logger = Box::new(WebLoggerService {
    enable_debug: true,
    enable_error: true,
    enable_warn: true,
    enable_info: true,
    location: vec![],
    target: Box::new(LocalConsoleLogger {}),
  });

  let input_logger = logger.create_sublogger("input");
  let (input_service, listener) = WebInputService::create(window.clone(), input_logger);
  listener.bootstrap(&pre_node, &window)?;

  std::mem::forget(listener);

  Ok(DemoGameLoop {
    game: Game::new(logger.create_sublogger("game")),
    input_service: Box::new(input_service),
    logger,
    text_node,
  })
}

mod util {
  use mythos_web::base::css::Unit;
  use mythos_web::base::element::{CreateElement, ElementFactory};
  use wasm_bindgen::JsCast;
  use web_sys::{Document, HtmlElement, window, Window};
  use super::error::DemoError;

  pub fn create_element<T: JsCast>(name: &str) -> Result<T, DemoError> {
    let create_element = get_document().map(CreateElement::new)?;
    create_element.factory::<T>(name).build().map_err(|e| e.into())
  }

  pub fn style_page() -> Result<(), DemoError> {
    let html = get_html()?;
    ElementFactory::new_from_element(html)
      .set_style("height", Unit::percent_u(100))
      .build()?;

    let body = get_body()?;
    ElementFactory::new_from_element(body)
      .set_style("margin", Unit::px_u(0))
      .set_style("height", Unit::percent_u(100))
      .set_style_with_str("display", "flex")
      .set_style_with_str("align-items", "center")
      .set_style_with_str("justify-content", "center")
      .build()?;
    Ok(())
  }

  pub fn get_html() -> Result<HtmlElement, DemoError> {
    get_body()?
      .parent_node()
      .ok_or(DemoError::CannotFindHtml)?
      .dyn_into::<HtmlElement>()
      .map_err(|_| DemoError::CannotFindHtml)
  }

  pub fn get_body() -> Result<HtmlElement, DemoError> {
    get_document()?.body().ok_or(DemoError::CannotFindBody)
  }

  pub fn get_document() -> Result<Document, DemoError> {
    get_window()?
      .document()
      .ok_or(DemoError::CannotFindDocument)
  }

  pub fn get_window() -> Result<Window, DemoError> {
    window().ok_or(DemoError::CannotFindWindow)
  }
}

mod error {
  use mythos_web::base::element::CreateElementError;
  use mythos_web::bindings::input::WebInputServiceError;
  use std::error::Error;
  use wasm_bindgen::JsValue;

  #[derive(Debug)]
  pub enum DemoError {
    CannotFindBody,
    CannotFindDocument,
    CannotFindHtml,
    CannotFindWindow,
    Js(JsValue),
    ElementFactory(CreateElementError),
    WebInputService(WebInputServiceError),
  }

  impl From<CreateElementError> for DemoError {
    fn from(value: CreateElementError) -> Self {
      DemoError::ElementFactory(value)
    }
  }

  impl From<WebInputServiceError> for DemoError {
    fn from(value: WebInputServiceError) -> Self {
      DemoError::WebInputService(value)
    }
  }

  impl From<JsValue> for DemoError {
    fn from(value: JsValue) -> Self {
      DemoError::Js(value)
    }
  }

  impl std::fmt::Display for DemoError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      match self {
        DemoError::Js(e) => write!(formatter, "A Js error occured, {:?}", e),
        DemoError::CannotFindBody => write!(formatter, "No body found"),
        DemoError::CannotFindDocument => write!(formatter, "No document found"),
        DemoError::CannotFindHtml => write!(formatter, "No html element found"),
        DemoError::CannotFindWindow => write!(formatter, "No window found"),
        DemoError::ElementFactory(e) => write!(formatter, "GameLoop -> {}", e),
        DemoError::WebInputService(e) => {
          write!(formatter, "WebInputService, {}", e)
        },
      }
    }
  }

  impl Error for DemoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
      None
    }
  }
}
