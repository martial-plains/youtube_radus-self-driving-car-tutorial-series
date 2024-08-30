use std::{cell::RefCell, rc::Rc};

use gloo::utils::document;
use js_sys::Function;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::KeyboardEvent;

pub type ControlsPtr = Rc<RefCell<Controls>>;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ControlKind {
    Keys,
    Dummy,
    AI,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Controls {
    pub forward: bool,
    pub left: bool,
    pub right: bool,
    pub reverse: bool,
}

impl Controls {
    pub fn new(kind: ControlKind) -> ControlsPtr {
        let controls = Self::default();

        let controls_ptr = Rc::new(RefCell::new(controls));

        match kind {
            ControlKind::Keys => Self::add_keyboard_listeners(&controls_ptr),
            ControlKind::Dummy => controls_ptr.borrow_mut().forward = true,
            ControlKind::AI => (),
        }

        controls_ptr
    }

    fn add_keyboard_listeners(this: &ControlsPtr) {
        let keydown = {
            let this = this.clone();
            Closure::<dyn FnMut(KeyboardEvent)>::new(move |event: KeyboardEvent| {
                if let Ok(mut this) = this.try_borrow_mut() {
                    match event.key().as_str() {
                        "ArrowLeft" => this.left = true,
                        "ArrowRight" => this.right = true,
                        "ArrowUp" => this.forward = true,
                        "ArrowDown" => this.reverse = true,
                        _ => (),
                    }
                }
            })
            .into_js_value()
            .dyn_into::<Function>()
            .unwrap()
        };

        document()
            .add_event_listener_with_callback("keydown", &keydown)
            .unwrap();

        let keyup = {
            let this = this.clone();
            Closure::<dyn FnMut(KeyboardEvent)>::new(move |event: KeyboardEvent| {
                if let Ok(mut this) = this.try_borrow_mut() {
                    match event.key().as_str() {
                        "ArrowLeft" => this.left = false,
                        "ArrowRight" => this.right = false,
                        "ArrowUp" => this.forward = false,
                        "ArrowDown" => this.reverse = false,
                        _ => (),
                    }
                }
            })
            .into_js_value()
            .dyn_into::<Function>()
            .unwrap()
        };

        document()
            .add_event_listener_with_callback("keyup", &keyup)
            .unwrap();
    }
}
