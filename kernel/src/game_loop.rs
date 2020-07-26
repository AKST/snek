use mythos_web::game_loop::*;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlPreElement, Text};

pub struct DemoGameLoop {
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
    Ok(DemoGameLoop::initialise()?)
  }

  fn install<'a>(& 'a mut self) -> AsyncGameLoopResult<'a, ()> {
    Box::pin(self.install_deps())
  }

  fn on_animation_frame(&mut self, t: f64) -> GameLoopResult<()> {
    let message = format!("tick {}", t);
    self.text_node.set_data(&message);
    Ok(())
  }

  fn on_init_error(error: &dyn std::error::Error) -> String {
    format!("{}", error)
  }

  fn on_error(&self, error: &dyn std::error::Error) {
    console_error(&Self::on_init_error(error));
  }
}

impl DemoGameLoop {
  fn initialise() -> Result<Self, error::DemoError> {
    util::style_page()?;

    let body = util::get_body()?;
    let pre_node = util::create_element::<HtmlPreElement>("pre")?;
    let text_node = Text::new()?;
    pre_node.append_child(&text_node)?;
    body.append_child(&pre_node)?;
    Ok(DemoGameLoop { text_node })
  }

  async fn install_deps(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
  }
}

mod util {
  use mythos_web::base::css::Unit;
  use mythos_web::base::element::{CreateElement, ElementFactory};
  use wasm_bindgen::JsCast;
  use web_sys::{Document, HtmlElement, window};
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
    window()
      .and_then(|w| w.document())
      .ok_or(DemoError::CannotFindDocument)
  }
}

mod error {
  use mythos_web::base::element::CreateElementError;
  use std::error::Error;
  use wasm_bindgen::JsValue;

  #[derive(Debug)]
  pub enum DemoError {
    CannotFindBody,
    CannotFindDocument,
    CannotFindHtml,
    Js(JsValue),
    ElementFactory(CreateElementError),
  }

  impl From<CreateElementError> for DemoError {
    fn from(value: CreateElementError) -> Self {
      DemoError::ElementFactory(value)
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
        DemoError::ElementFactory(e) => write!(formatter, "GameLoop -> {}", e),
      }
    }
  }

  impl Error for DemoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
      None
    }
  }
}
