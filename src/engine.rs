pub mod audio;
pub mod game_loop;
pub mod image;
pub mod key_state;
pub mod point;
pub mod rect;
pub mod renderer;
pub mod sprite_sheet;

pub use game_loop::{Game, GameLoop};
pub use image::Image;
pub use key_state::KeyState;
pub use point::Point;
pub use rect::Rect;
pub use renderer::Renderer;
pub use sprite_sheet::{Cell, SpriteSheet};

use std::{cell::RefCell, rc::Rc, sync::Mutex};

use anyhow::{anyhow, Result};
use futures::channel::{
    mpsc::{unbounded, UnboundedReceiver},
    oneshot::channel,
};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{HtmlElement, HtmlImageElement};

use crate::browser;

const FRAME_SIZE: f32 = 1.0 / 60.0 * 1000.0;

pub async fn load_image(source: &str) -> Result<HtmlImageElement> {
    let image = browser::new_image()?;

    let (complete_tx, complete_rx) = channel::<Result<()>>();
    let success_tx = Rc::new(Mutex::new(Some(complete_tx)));
    let error_tx = Rc::clone(&success_tx);
    let success_callback = browser::closure_once(move || {
        if let Some(success_tx) = success_tx.lock().ok().and_then(|mut opt| opt.take()) {
            if let Err(err) = success_tx.send(Ok(())) {
                error!(
                    "Unable to send success message after loading image {:#?}",
                    err
                );
            }
        }
    });
    let error_callback: Closure<dyn FnMut(JsValue)> = browser::closure_once(move |err| {
        if let Some(error_tx) = error_tx.lock().ok().and_then(|mut opt| opt.take()) {
            if let Err(err) = error_tx.send(Err(anyhow!("Error loading image: {:#?}", err))) {
                error!(
                    "Unable to send error message after trying to load image {:#?}",
                    err
                );
            }
        }
    });

    image.set_onload(Some(success_callback.as_ref().unchecked_ref()));
    image.set_onerror(Some(error_callback.as_ref().unchecked_ref()));
    image.set_src(source);

    complete_rx.await??;

    Ok(image)
}

enum KeyPress {
    KeyUp(web_sys::KeyboardEvent),
    KeyDown(web_sys::KeyboardEvent),
}

fn prepare_input() -> Result<UnboundedReceiver<KeyPress>> {
    let (keyevent_sender, keyevent_receiver) = unbounded();
    let keyup_sender = Rc::new(RefCell::new(keyevent_sender));
    let keydown_sender = Rc::clone(&keyup_sender);

    let onkeydown = browser::closure_wrap(Box::new(move |keycode: web_sys::KeyboardEvent| {
        if let Err(err) = keydown_sender
            .borrow_mut()
            .start_send(KeyPress::KeyDown(keycode))
        {
            error!("Could not send key down event {:#?}", err);
        }
    }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

    let onkeyup = browser::closure_wrap(Box::new(move |keycode: web_sys::KeyboardEvent| {
        if let Err(err) = keyup_sender
            .borrow_mut()
            .start_send(KeyPress::KeyUp(keycode))
        {
            error!("Could not send key up event {:#?}", err);
        }
    }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

    browser::canvas()
        .unwrap()
        .set_onkeydown(Some(onkeydown.as_ref().unchecked_ref()));
    browser::canvas()
        .unwrap()
        .set_onkeyup(Some(onkeyup.as_ref().unchecked_ref()));
    onkeydown.forget();
    onkeyup.forget();

    Ok(keyevent_receiver)
}

fn process_input(state: &mut KeyState, keyevent_receiver: &mut UnboundedReceiver<KeyPress>) {
    loop {
        match keyevent_receiver.try_next() {
            Ok(None) => break,
            Err(_err) => break,
            Ok(Some(evt)) => match evt {
                KeyPress::KeyDown(evt) => state.set_pressed(&evt.code(), evt),
                KeyPress::KeyUp(evt) => state.set_released(&evt.code()),
            },
        }
    }
}

pub fn add_click_handler(elem: HtmlElement) -> UnboundedReceiver<()> {
    let (mut click_sender, click_receiver) = unbounded();
    let on_click = browser::closure_wrap(Box::new(move || {
        if let Err(err) = click_sender.start_send(()) {
            error!("Could not send click event {:#?}", err);
        }
    }) as Box<dyn FnMut()>);
    elem.set_onclick(Some(on_click.as_ref().unchecked_ref()));
    on_click.forget();
    click_receiver
}

#[allow(dead_code)]
unsafe fn draw_frame_rate(renderer: &Renderer, frame_time: f64) {
    static mut FRAMES_COUNTED: i32 = 0;
    static mut TOTAL_FRAME_TIME: f64 = 0.0;
    static mut FRAME_RATE: i32 = 0;

    FRAMES_COUNTED += 1;
    TOTAL_FRAME_TIME += frame_time;

    if TOTAL_FRAME_TIME > 1000.0 {
        FRAME_RATE = FRAMES_COUNTED;
        TOTAL_FRAME_TIME = 0.0;
        FRAMES_COUNTED = 0;
    }

    if let Err(err) = renderer.draw_text(
        &format!("Frame rate: {}", FRAME_RATE),
        &Point { x: 400, y: 100 },
    ) {
        error!("Could not draw text {:#?}", err);
    }
}
