use crate::{
    AnyWindowHandle, Bounds, Capslock, DevicePixels, DispatchEventResult, GpuSpecs, Keystroke,
    ModifiersChangedEvent, Modifiers, MouseButton, MouseDownEvent, MouseExitEvent, MouseMoveEvent,
    MouseUpEvent, Pixels, PlatformAtlas, PlatformDisplay, PlatformInput, PlatformInputHandler,
    PlatformWindow, Point, PromptButton, PromptLevel, RequestFrameOptions, Scene, ScrollDelta,
    ScrollWheelEvent, Size, TouchPhase, WindowAppearance, WindowBackgroundAppearance, WindowBounds,
    WindowParams, px, KeyDownEvent, KeyUpEvent,
};
use anyhow::Result;
use futures::channel::oneshot;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WebDisplayHandle, WebWindowHandle, WindowHandle,
};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::Arc,
};

use super::{WgpuAtlas, WgpuRenderer};

pub(crate) struct WebWindowState {
    handle: AnyWindowHandle,
    canvas_id: u32,
    canvas: Option<web_sys::HtmlCanvasElement>,
    ime_input: Option<web_sys::HtmlInputElement>,
    renderer: Option<WgpuRenderer>,
    size: Size<Pixels>,
    scale_factor: f32,
    input_handler: Option<PlatformInputHandler>,
    request_frame_callback: Option<Box<dyn FnMut(RequestFrameOptions)>>,
    input_callback: Option<Box<dyn FnMut(PlatformInput) -> DispatchEventResult>>,
    resize_callback: Option<Box<dyn FnMut(Size<Pixels>, f32)>>,
    active_callback: Option<Box<dyn FnMut(bool)>>,
    hover_callback: Option<Box<dyn FnMut(bool)>>,
    close_callback: Option<Box<dyn FnOnce()>>,
    appearance_changed_callback: Option<Box<dyn FnMut()>>,
    needs_force_render: bool,
    mouse_position: Point<Pixels>,
    modifiers: Modifiers,
    capslock: Capslock,
    pressed_button: Option<MouseButton>,
    is_hovered: bool,
    click_count: usize,
    last_click_time: f64,
    last_click_position: Point<Pixels>,
    is_composing: bool,
}

struct WebWindowInner(RefCell<WebWindowState>);

pub(crate) struct WebWindow(Rc<WebWindowInner>);

impl Clone for WebWindow {
    fn clone(&self) -> Self {
        WebWindow(self.0.clone())
    }
}

impl WebWindow {
    pub fn new(handle: AnyWindowHandle, params: WindowParams) -> Result<Self> {
        static NEXT_CANVAS_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
        let canvas_id = NEXT_CANVAS_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let scale_factor = web_sys::window()
            .map(|w| w.device_pixel_ratio() as f32)
            .unwrap_or(1.0);

        let size = params.bounds.size;

        Ok(Self(Rc::new(WebWindowInner(RefCell::new(WebWindowState {
            handle,
            canvas_id,
            canvas: None,
            ime_input: None,
            renderer: None,
            size,
            scale_factor,
            input_handler: None,
            request_frame_callback: None,
            input_callback: None,
            resize_callback: None,
            active_callback: None,
            hover_callback: None,
            close_callback: None,
            appearance_changed_callback: None,
            needs_force_render: false,
            mouse_position: Point::default(),
            modifiers: Modifiers::default(),
            capslock: Capslock::default(),
            pressed_button: None,
            is_hovered: false,
            click_count: 0,
            last_click_time: 0.0,
            last_click_position: Point::default(),
            is_composing: false,
        })))))
    }

    pub fn canvas_id(&self) -> u32 {
        self.0.0.borrow().canvas_id
    }

    pub async fn initialize_renderer(&self) -> Result<()> {
        let (canvas_id, size, scale_factor) = {
            let state = self.0.0.borrow();
            (state.canvas_id, state.size, state.scale_factor)
        };

        let document = web_sys::window()
            .and_then(|w| w.document())
            .ok_or_else(|| anyhow::anyhow!("No document"))?;

        let canvas = document
            .create_element("canvas")
            .map_err(|_| anyhow::anyhow!("Failed to create canvas"))?
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| anyhow::anyhow!("Element is not a canvas"))?;

        canvas.set_attribute("data-raw-handle", &canvas_id.to_string())
            .map_err(|_| anyhow::anyhow!("Failed to set canvas attribute"))?;

        let device_size = Size {
            width: DevicePixels((size.width.0 * scale_factor) as i32),
            height: DevicePixels((size.height.0 * scale_factor) as i32),
        };

        canvas.set_width(device_size.width.0 as u32);
        canvas.set_height(device_size.height.0 as u32);

        let style = canvas.style();
        style.set_property("width", &format!("{}px", size.width.0))
            .map_err(|_| anyhow::anyhow!("Failed to set canvas width"))?;
        style.set_property("height", &format!("{}px", size.height.0))
            .map_err(|_| anyhow::anyhow!("Failed to set canvas height"))?;

        // Create hidden IME input element for composition support
        let ime_input = document
            .create_element("input")
            .map_err(|_| anyhow::anyhow!("Failed to create IME input"))?
            .dyn_into::<web_sys::HtmlInputElement>()
            .map_err(|_| anyhow::anyhow!("Element is not an input"))?;

        ime_input.set_type("text");
        ime_input.set_id(&format!("gpui-ime-input-{}", canvas_id));

        let ime_style = ime_input.style();
        let _ = ime_style.set_property("position", "absolute");
        let _ = ime_style.set_property("opacity", "0");
        let _ = ime_style.set_property("pointer-events", "none");
        let _ = ime_style.set_property("width", "1px");
        let _ = ime_style.set_property("height", "1px");
        let _ = ime_style.set_property("left", "0");
        let _ = ime_style.set_property("top", "0");
        let _ = ime_style.set_property("z-index", "-1");

        if let Some(body) = document.body() {
            body.append_child(&canvas)
                .map_err(|_| anyhow::anyhow!("Failed to append canvas to body"))?;
            body.append_child(&ime_input)
                .map_err(|_| anyhow::anyhow!("Failed to append IME input to body"))?;
        }

        let renderer = WgpuRenderer::new(canvas.clone(), device_size, false).await?;

        self.setup_event_listeners(&canvas);
        self.setup_ime_listeners(&ime_input);

        {
            let mut state = self.0.0.borrow_mut();
            state.canvas = Some(canvas);
            state.ime_input = Some(ime_input);
            state.renderer = Some(renderer);
            state.needs_force_render = true;
        }

        // Start the render loop
        self.start_render_loop();

        Ok(())
    }

    fn start_render_loop(&self) {
        let window = self.clone();
        let closure = Rc::new(RefCell::new(None::<wasm_bindgen::closure::Closure<dyn FnMut()>>));
        let closure_clone = closure.clone();

        *closure.borrow_mut() = Some(wasm_bindgen::closure::Closure::new(move || {
            // Call the frame callback if set
            {
                let mut state = window.0.0.borrow_mut();
                if let Some(mut callback) = state.request_frame_callback.take() {
                    let force_render = state.needs_force_render;
                    if force_render {
                        state.needs_force_render = false;
                    }
                    drop(state);
                    callback(RequestFrameOptions {
                        force_render,
                        ..Default::default()
                    });
                    window.0.0.borrow_mut().request_frame_callback = Some(callback);
                }
            }

            // Schedule the next frame
            if let Some(win) = web_sys::window() {
                let closure_ref = closure_clone.borrow();
                if let Some(c) = closure_ref.as_ref() {
                    let _ = win.request_animation_frame(c.as_ref().unchecked_ref());
                }
            }
        }));

        // Start the loop
        if let Some(win) = web_sys::window() {
            let closure_ref = closure.borrow();
            if let Some(c) = closure_ref.as_ref() {
                let _ = win.request_animation_frame(c.as_ref().unchecked_ref());
            }
        }

        // Store the closure to prevent it from being dropped
        // Note: This leaks the closure, but that's okay for a render loop
        std::mem::forget(closure);
    }

    pub fn sprite_atlas(&self) -> Option<Arc<WgpuAtlas>> {
        self.0.0.borrow().renderer.as_ref().map(|r| r.sprite_atlas().clone())
    }

    fn setup_event_listeners(&self, canvas: &web_sys::HtmlCanvasElement) {
        use wasm_bindgen::closure::Closure;

        // Mouse move listener
        {
            let window = self.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
                let position = Point {
                    x: px(event.offset_x() as f32),
                    y: px(event.offset_y() as f32),
                };
                let modifiers = modifiers_from_mouse_event(&event);

                let mut state = window.0.0.borrow_mut();
                state.mouse_position = position;
                state.modifiers = modifiers;
                let pressed_button = state.pressed_button;

                if let Some(mut callback) = state.input_callback.take() {
                    drop(state);
                    let event = PlatformInput::MouseMove(MouseMoveEvent {
                        position,
                        pressed_button,
                        modifiers,
                    });
                    callback(event);
                    window.0.0.borrow_mut().input_callback = Some(callback);
                }
            });
            canvas
                .add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();
        }

        // Mouse down listener
        {
            let window = self.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
                event.prevent_default();
                let position = Point {
                    x: px(event.offset_x() as f32),
                    y: px(event.offset_y() as f32),
                };
                let modifiers = modifiers_from_mouse_event(&event);
                let button = mouse_button_from_web(event.button());

                let mut state = window.0.0.borrow_mut();
                state.mouse_position = position;
                state.modifiers = modifiers;
                state.pressed_button = Some(button);

                let now = js_sys::Date::now();
                let click_count = if now - state.last_click_time < 500.0
                    && (state.last_click_position.x - position.x).0.abs() < 5.0
                    && (state.last_click_position.y - position.y).0.abs() < 5.0
                {
                    state.click_count + 1
                } else {
                    1
                };
                state.click_count = click_count;
                state.last_click_time = now;
                state.last_click_position = position;

                if let Some(mut callback) = state.input_callback.take() {
                    drop(state);
                    let event = PlatformInput::MouseDown(MouseDownEvent {
                        button,
                        position,
                        modifiers,
                        click_count,
                        first_mouse: false,
                    });
                    callback(event);
                    window.0.0.borrow_mut().input_callback = Some(callback);
                }
            });
            canvas
                .add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();
        }

        // Mouse up listener
        {
            let window = self.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
                let position = Point {
                    x: px(event.offset_x() as f32),
                    y: px(event.offset_y() as f32),
                };
                let modifiers = modifiers_from_mouse_event(&event);
                let button = mouse_button_from_web(event.button());

                let mut state = window.0.0.borrow_mut();
                state.mouse_position = position;
                state.modifiers = modifiers;
                state.pressed_button = None;
                let click_count = state.click_count;

                if let Some(mut callback) = state.input_callback.take() {
                    drop(state);
                    let event = PlatformInput::MouseUp(MouseUpEvent {
                        button,
                        position,
                        modifiers,
                        click_count,
                    });
                    callback(event);
                    window.0.0.borrow_mut().input_callback = Some(callback);
                }
            });
            canvas
                .add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();
        }

        // Mouse enter listener (for hover tracking)
        {
            let window = self.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::MouseEvent| {
                let mut state = window.0.0.borrow_mut();
                if !state.is_hovered {
                    state.is_hovered = true;
                    if let Some(mut callback) = state.hover_callback.take() {
                        drop(state);
                        callback(true);
                        window.0.0.borrow_mut().hover_callback = Some(callback);
                    }
                }
            });
            canvas
                .add_event_listener_with_callback("mouseenter", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();
        }

        // Mouse leave listener (for hover tracking and mouse exit event)
        {
            let window = self.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
                let position = Point {
                    x: px(event.offset_x() as f32),
                    y: px(event.offset_y() as f32),
                };
                let modifiers = modifiers_from_mouse_event(&event);

                let mut state = window.0.0.borrow_mut();
                state.is_hovered = false;
                let pressed_button = state.pressed_button.take();
                state.modifiers = modifiers;

                let exit_event = MouseExitEvent {
                    position,
                    pressed_button,
                    modifiers,
                };

                if let Some(mut hover_callback) = state.hover_callback.take() {
                    let input_callback = state.input_callback.take();
                    drop(state);
                    hover_callback(false);

                    if let Some(mut input_cb) = input_callback {
                        input_cb(PlatformInput::MouseExited(exit_event));
                        window.0.0.borrow_mut().input_callback = Some(input_cb);
                    }
                    window.0.0.borrow_mut().hover_callback = Some(hover_callback);
                } else if let Some(mut callback) = state.input_callback.take() {
                    drop(state);
                    callback(PlatformInput::MouseExited(exit_event));
                    window.0.0.borrow_mut().input_callback = Some(callback);
                }
            });
            canvas
                .add_event_listener_with_callback("mouseleave", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();
        }

        // Wheel listener for scroll events
        {
            let window = self.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::WheelEvent| {
                event.prevent_default();
                let position = Point {
                    x: px(event.offset_x() as f32),
                    y: px(event.offset_y() as f32),
                };
                let modifiers = modifiers_from_mouse_event(&event);

                let delta_mode = event.delta_mode();
                let delta = if delta_mode == web_sys::WheelEvent::DOM_DELTA_PIXEL {
                    ScrollDelta::Pixels(Point {
                        x: px(-event.delta_x() as f32),
                        y: px(-event.delta_y() as f32),
                    })
                } else {
                    ScrollDelta::Lines(Point {
                        x: -event.delta_x() as f32,
                        y: -event.delta_y() as f32,
                    })
                };

                let mut state = window.0.0.borrow_mut();
                state.modifiers = modifiers;

                if let Some(mut callback) = state.input_callback.take() {
                    drop(state);
                    let event = PlatformInput::ScrollWheel(ScrollWheelEvent {
                        position,
                        delta,
                        modifiers,
                        touch_phase: TouchPhase::Moved,
                    });
                    callback(event);
                    window.0.0.borrow_mut().input_callback = Some(callback);
                }
            });
            canvas
                .add_event_listener_with_callback("wheel", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();
        }

        // Context menu prevention (right-click) - we handle right-click via mousedown instead
        {
            let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
                event.prevent_default();
            });
            canvas
                .add_event_listener_with_callback("contextmenu", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();
        }

        // Touch start listener
        {
            let window = self.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::TouchEvent| {
                event.prevent_default();
                let touches = event.changed_touches();
                if touches.length() == 0 {
                    return;
                }
                let touch = match touches.get(0) {
                    Some(t) => t,
                    None => return,
                };

                let canvas_rect = {
                    let state = window.0.0.borrow();
                    state.canvas.as_ref().map(|c| c.get_bounding_client_rect())
                };
                let (offset_x, offset_y) = if let Some(rect) = canvas_rect {
                    (touch.client_x() as f32 - rect.left() as f32, touch.client_y() as f32 - rect.top() as f32)
                } else {
                    (touch.client_x() as f32, touch.client_y() as f32)
                };

                let position = Point {
                    x: px(offset_x),
                    y: px(offset_y),
                };
                let modifiers = Modifiers::default();

                let mut state = window.0.0.borrow_mut();
                state.mouse_position = position;
                state.pressed_button = Some(MouseButton::Left);

                let now = js_sys::Date::now();
                let click_count = if now - state.last_click_time < 500.0
                    && (state.last_click_position.x - position.x).0.abs() < 20.0
                    && (state.last_click_position.y - position.y).0.abs() < 20.0
                {
                    state.click_count + 1
                } else {
                    1
                };
                state.click_count = click_count;
                state.last_click_time = now;
                state.last_click_position = position;

                if let Some(mut callback) = state.input_callback.take() {
                    drop(state);
                    let event = PlatformInput::MouseDown(MouseDownEvent {
                        button: MouseButton::Left,
                        position,
                        modifiers,
                        click_count,
                        first_mouse: false,
                    });
                    callback(event);
                    window.0.0.borrow_mut().input_callback = Some(callback);
                }
            });
            canvas
                .add_event_listener_with_callback("touchstart", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();
        }

        // Touch move listener
        {
            let window = self.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::TouchEvent| {
                event.prevent_default();
                let touches = event.changed_touches();
                if touches.length() == 0 {
                    return;
                }
                let touch = match touches.get(0) {
                    Some(t) => t,
                    None => return,
                };

                let canvas_rect = {
                    let state = window.0.0.borrow();
                    state.canvas.as_ref().map(|c| c.get_bounding_client_rect())
                };
                let (offset_x, offset_y) = if let Some(rect) = canvas_rect {
                    (touch.client_x() as f32 - rect.left() as f32, touch.client_y() as f32 - rect.top() as f32)
                } else {
                    (touch.client_x() as f32, touch.client_y() as f32)
                };

                let position = Point {
                    x: px(offset_x),
                    y: px(offset_y),
                };
                let modifiers = Modifiers::default();

                let mut state = window.0.0.borrow_mut();
                state.mouse_position = position;

                if let Some(mut callback) = state.input_callback.take() {
                    drop(state);
                    let event = PlatformInput::MouseMove(MouseMoveEvent {
                        position,
                        pressed_button: Some(MouseButton::Left),
                        modifiers,
                    });
                    callback(event);
                    window.0.0.borrow_mut().input_callback = Some(callback);
                }
            });
            canvas
                .add_event_listener_with_callback("touchmove", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();
        }

        // Touch end listener
        {
            let window = self.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::TouchEvent| {
                event.prevent_default();
                let touches = event.changed_touches();
                if touches.length() == 0 {
                    return;
                }
                let touch = match touches.get(0) {
                    Some(t) => t,
                    None => return,
                };

                let canvas_rect = {
                    let state = window.0.0.borrow();
                    state.canvas.as_ref().map(|c| c.get_bounding_client_rect())
                };
                let (offset_x, offset_y) = if let Some(rect) = canvas_rect {
                    (touch.client_x() as f32 - rect.left() as f32, touch.client_y() as f32 - rect.top() as f32)
                } else {
                    (touch.client_x() as f32, touch.client_y() as f32)
                };

                let position = Point {
                    x: px(offset_x),
                    y: px(offset_y),
                };
                let modifiers = Modifiers::default();

                let mut state = window.0.0.borrow_mut();
                state.mouse_position = position;
                state.pressed_button = None;
                let click_count = state.click_count;

                if let Some(mut callback) = state.input_callback.take() {
                    drop(state);
                    let event = PlatformInput::MouseUp(MouseUpEvent {
                        button: MouseButton::Left,
                        position,
                        modifiers,
                        click_count,
                    });
                    callback(event);
                    window.0.0.borrow_mut().input_callback = Some(callback);
                }
            });
            canvas
                .add_event_listener_with_callback("touchend", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();
        }

        // Touch cancel listener (treat like touch end)
        {
            let window = self.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::TouchEvent| {
                let touches = event.changed_touches();
                if touches.length() == 0 {
                    return;
                }
                let touch = match touches.get(0) {
                    Some(t) => t,
                    None => return,
                };

                let canvas_rect = {
                    let state = window.0.0.borrow();
                    state.canvas.as_ref().map(|c| c.get_bounding_client_rect())
                };
                let (offset_x, offset_y) = if let Some(rect) = canvas_rect {
                    (touch.client_x() as f32 - rect.left() as f32, touch.client_y() as f32 - rect.top() as f32)
                } else {
                    (touch.client_x() as f32, touch.client_y() as f32)
                };

                let position = Point {
                    x: px(offset_x),
                    y: px(offset_y),
                };
                let modifiers = Modifiers::default();

                let mut state = window.0.0.borrow_mut();
                state.mouse_position = position;
                state.pressed_button = None;
                let click_count = state.click_count;

                if let Some(mut callback) = state.input_callback.take() {
                    drop(state);
                    let event = PlatformInput::MouseUp(MouseUpEvent {
                        button: MouseButton::Left,
                        position,
                        modifiers,
                        click_count,
                    });
                    callback(event);
                    window.0.0.borrow_mut().input_callback = Some(callback);
                }
            });
            canvas
                .add_event_listener_with_callback("touchcancel", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();
        }

        // Keyboard events on the window
        if let Some(browser_window) = web_sys::window() {
            // Key down listener
            {
                let window = self.clone();
                let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::KeyboardEvent| {
                    let modifiers = modifiers_from_keyboard_event(&event);
                    let capslock = Capslock {
                        on: event.get_modifier_state("CapsLock"),
                    };
                    let key = key_from_web_event(&event);

                    let mut state = window.0.0.borrow_mut();
                    let old_modifiers = state.modifiers;
                    let old_capslock = state.capslock;
                    let is_composing = state.is_composing || event.is_composing();
                    state.modifiers = modifiers;
                    state.capslock = capslock;

                    if let Some(mut callback) = state.input_callback.take() {
                        drop(state);

                        if old_modifiers != modifiers || old_capslock != capslock {
                            callback(PlatformInput::ModifiersChanged(ModifiersChangedEvent {
                                modifiers,
                                capslock,
                            }));
                        }

                        if !is_composing && !is_modifier_key(&event) {
                            let keystroke = Keystroke {
                                modifiers,
                                key: key.clone(),
                                key_char: event.key().chars().next().filter(|c| !c.is_control()).map(|c| c.to_string()),
                            };
                            callback(PlatformInput::KeyDown(KeyDownEvent {
                                keystroke,
                                is_held: event.repeat(),
                                prefer_character_input: false,
                            }));
                        }

                        window.0.0.borrow_mut().input_callback = Some(callback);
                    }
                });
                browser_window
                    .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
                    .ok();
                closure.forget();
            }

            // Key up listener
            {
                let window = self.clone();
                let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::KeyboardEvent| {
                    let modifiers = modifiers_from_keyboard_event(&event);
                    let capslock = Capslock {
                        on: event.get_modifier_state("CapsLock"),
                    };
                    let key = key_from_web_event(&event);

                    let mut state = window.0.0.borrow_mut();
                    let old_modifiers = state.modifiers;
                    let old_capslock = state.capslock;
                    let is_composing = state.is_composing || event.is_composing();
                    state.modifiers = modifiers;
                    state.capslock = capslock;

                    if let Some(mut callback) = state.input_callback.take() {
                        drop(state);

                        if !is_composing && !is_modifier_key(&event) {
                            let keystroke = Keystroke {
                                modifiers,
                                key,
                                key_char: None,
                            };
                            callback(PlatformInput::KeyUp(KeyUpEvent { keystroke }));
                        }

                        if old_modifiers != modifiers || old_capslock != capslock {
                            callback(PlatformInput::ModifiersChanged(ModifiersChangedEvent {
                                modifiers,
                                capslock,
                            }));
                        }

                        window.0.0.borrow_mut().input_callback = Some(callback);
                    }
                });
                browser_window
                    .add_event_listener_with_callback("keyup", closure.as_ref().unchecked_ref())
                    .ok();
                closure.forget();
            }

            // Focus listener (window becomes active)
            {
                let window = self.clone();
                let closure = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::FocusEvent| {
                    let mut state = window.0.0.borrow_mut();
                    if let Some(mut callback) = state.active_callback.take() {
                        drop(state);
                        callback(true);
                        window.0.0.borrow_mut().active_callback = Some(callback);
                    }
                });
                browser_window
                    .add_event_listener_with_callback("focus", closure.as_ref().unchecked_ref())
                    .ok();
                closure.forget();
            }

            // Blur listener (window becomes inactive)
            {
                let window = self.clone();
                let closure = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::FocusEvent| {
                    let mut state = window.0.0.borrow_mut();
                    if let Some(mut callback) = state.active_callback.take() {
                        drop(state);
                        callback(false);
                        window.0.0.borrow_mut().active_callback = Some(callback);
                    }
                });
                browser_window
                    .add_event_listener_with_callback("blur", closure.as_ref().unchecked_ref())
                    .ok();
                closure.forget();
            }

            // Visibility change listener (tab hidden/shown)
            {
                let window = self.clone();
                let closure = Closure::<dyn FnMut()>::new(move || {
                    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                        let is_visible = !document.hidden();
                        let mut state = window.0.0.borrow_mut();
                        if let Some(mut callback) = state.active_callback.take() {
                            drop(state);
                            callback(is_visible);
                            window.0.0.borrow_mut().active_callback = Some(callback);
                        }
                    }
                });
                if let Some(document) = browser_window.document() {
                    document
                        .add_event_listener_with_callback("visibilitychange", closure.as_ref().unchecked_ref())
                        .ok();
                }
                closure.forget();
            }

            // Appearance change listener (dark/light mode)
            if let Ok(Some(media_query)) = browser_window.match_media("(prefers-color-scheme: dark)") {
                let window = self.clone();
                let closure = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::MediaQueryListEvent| {
                    let mut state = window.0.0.borrow_mut();
                    if let Some(mut callback) = state.appearance_changed_callback.take() {
                        drop(state);
                        callback();
                        window.0.0.borrow_mut().appearance_changed_callback = Some(callback);
                    }
                });
                media_query
                    .add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())
                    .ok();
                closure.forget();
            }

            // DPI change detection using matchMedia for devicePixelRatio
            // We create a media query for the current DPI and listen for changes
            let current_dpr = browser_window.device_pixel_ratio();
            let media_query_str = format!("(resolution: {}dppx)", current_dpr);
            if let Ok(Some(media_query)) = browser_window.match_media(&media_query_str) {
                let window = self.clone();
                let closure = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::MediaQueryListEvent| {
                    if let Some(browser_win) = web_sys::window() {
                        let new_scale_factor = browser_win.device_pixel_ratio() as f32;

                        let mut state = window.0.0.borrow_mut();
                        let old_scale_factor = state.scale_factor;

                        if (new_scale_factor - old_scale_factor).abs() > 0.001 {
                            state.scale_factor = new_scale_factor;
                            let size = state.size;

                            // Update canvas internal size for device pixels
                            if let Some(canvas) = &state.canvas {
                                let device_width = (size.width.0 * new_scale_factor) as u32;
                                let device_height = (size.height.0 * new_scale_factor) as u32;
                                canvas.set_width(device_width);
                                canvas.set_height(device_height);
                            }

                            // Update renderer
                            if let Some(renderer) = &mut state.renderer {
                                let device_size = Size {
                                    width: DevicePixels((size.width.0 * new_scale_factor) as i32),
                                    height: DevicePixels((size.height.0 * new_scale_factor) as i32),
                                };
                                renderer.update_drawable_size(device_size);
                            }

                            state.needs_force_render = true;

                            if let Some(mut callback) = state.resize_callback.take() {
                                drop(state);
                                callback(size, new_scale_factor);
                                window.0.0.borrow_mut().resize_callback = Some(callback);
                            }
                        }
                    }
                });
                media_query
                    .add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())
                    .ok();
                closure.forget();
            }
        }

        // ResizeObserver for canvas size changes
        {
            let window = self.clone();
            let closure = Closure::<dyn FnMut(js_sys::Array)>::new(move |entries: js_sys::Array| {
                if let Some(entry) = entries.get(0).dyn_ref::<web_sys::ResizeObserverEntry>() {
                    let content_rect = entry.content_rect();
                    let new_width = content_rect.width() as f32;
                    let new_height = content_rect.height() as f32;

                    let mut state = window.0.0.borrow_mut();
                    let scale_factor = state.scale_factor;
                    let new_size = Size {
                        width: px(new_width),
                        height: px(new_height),
                    };

                    if state.size != new_size {
                        state.size = new_size;

                        // Update canvas internal size for device pixels
                        if let Some(canvas) = &state.canvas {
                            let device_width = (new_width * scale_factor) as u32;
                            let device_height = (new_height * scale_factor) as u32;
                            canvas.set_width(device_width);
                            canvas.set_height(device_height);
                        }

                        // Update renderer
                        if let Some(renderer) = &mut state.renderer {
                            let device_size = Size {
                                width: DevicePixels((new_width * scale_factor) as i32),
                                height: DevicePixels((new_height * scale_factor) as i32),
                            };
                            renderer.update_drawable_size(device_size);
                        }

                        state.needs_force_render = true;

                        if let Some(mut callback) = state.resize_callback.take() {
                            drop(state);
                            callback(new_size, scale_factor);
                            window.0.0.borrow_mut().resize_callback = Some(callback);
                        }
                    }
                }
            });

            if let Ok(observer) = web_sys::ResizeObserver::new(closure.as_ref().unchecked_ref()) {
                observer.observe(canvas);
            }
            closure.forget();
        }
    }

    fn setup_ime_listeners(&self, ime_input: &web_sys::HtmlInputElement) {
        use wasm_bindgen::closure::Closure;

        // Composition start - marks the start of IME input
        {
            let window = self.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::CompositionEvent| {
                window.0.0.borrow_mut().is_composing = true;
            });
            ime_input
                .add_event_listener_with_callback("compositionstart", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();
        }

        // Composition update - called as the user is composing text
        {
            let window = self.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::CompositionEvent| {
                if let Some(data) = event.data() {
                    let mut state = window.0.0.borrow_mut();
                    if let Some(mut input_handler) = state.input_handler.take() {
                        drop(state);
                        input_handler.replace_and_mark_text_in_range(None, &data, None);
                        window.0.0.borrow_mut().input_handler = Some(input_handler);
                    }
                }
            });
            ime_input
                .add_event_listener_with_callback("compositionupdate", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();
        }

        // Composition end - final text committed
        {
            let window = self.clone();
            let ime_input_clone = ime_input.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::CompositionEvent| {
                let mut state = window.0.0.borrow_mut();
                state.is_composing = false;

                if let Some(data) = event.data() {
                    if let Some(mut input_handler) = state.input_handler.take() {
                        drop(state);
                        input_handler.replace_text_in_range(None, &data);
                        window.0.0.borrow_mut().input_handler = Some(input_handler);
                    }
                }

                ime_input_clone.set_value("");
            });
            ime_input
                .add_event_listener_with_callback("compositionend", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();
        }

        // Input event - handles direct text input (non-IME)
        {
            let window = self.clone();
            let ime_input_clone = ime_input.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::InputEvent| {
                let state = window.0.0.borrow();
                let is_composing = state.is_composing;
                drop(state);

                if is_composing {
                    return;
                }

                if let Some(data) = event.data() {
                    let mut state = window.0.0.borrow_mut();
                    if let Some(mut input_handler) = state.input_handler.take() {
                        drop(state);
                        input_handler.replace_text_in_range(None, &data);
                        window.0.0.borrow_mut().input_handler = Some(input_handler);
                    }
                }

                ime_input_clone.set_value("");
            });
            ime_input
                .add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();
        }
    }

    fn dispatch_input(&self, event: PlatformInput) {
        let mut state = self.0.0.borrow_mut();
        if let Some(mut callback) = state.input_callback.take() {
            drop(state);
            callback(event);
            self.0.0.borrow_mut().input_callback = Some(callback);
        }
    }
}

fn modifiers_from_mouse_event(event: &web_sys::MouseEvent) -> Modifiers {
    Modifiers {
        control: event.ctrl_key(),
        alt: event.alt_key(),
        shift: event.shift_key(),
        platform: event.meta_key(),
        function: false,
    }
}

fn modifiers_from_keyboard_event(event: &web_sys::KeyboardEvent) -> Modifiers {
    Modifiers {
        control: event.ctrl_key(),
        alt: event.alt_key(),
        shift: event.shift_key(),
        platform: event.meta_key(),
        function: event.get_modifier_state("Fn"),
    }
}

fn mouse_button_from_web(button: i16) -> MouseButton {
    match button {
        0 => MouseButton::Left,
        1 => MouseButton::Middle,
        2 => MouseButton::Right,
        3 => MouseButton::Navigate(crate::NavigationDirection::Back),
        4 => MouseButton::Navigate(crate::NavigationDirection::Forward),
        _ => MouseButton::Left,
    }
}

fn is_modifier_key(event: &web_sys::KeyboardEvent) -> bool {
    matches!(
        event.key().as_str(),
        "Shift" | "Control" | "Alt" | "Meta" | "CapsLock" | "Fn"
    )
}

fn key_from_web_event(event: &web_sys::KeyboardEvent) -> String {
    let key = event.key();
    match key.as_str() {
        "ArrowUp" => "up".to_string(),
        "ArrowDown" => "down".to_string(),
        "ArrowLeft" => "left".to_string(),
        "ArrowRight" => "right".to_string(),
        "Backspace" => "backspace".to_string(),
        "Delete" => "delete".to_string(),
        "Enter" => "enter".to_string(),
        "Tab" => "tab".to_string(),
        "Escape" => "escape".to_string(),
        "Home" => "home".to_string(),
        "End" => "end".to_string(),
        "PageUp" => "pageup".to_string(),
        "PageDown" => "pagedown".to_string(),
        " " => "space".to_string(),
        "F1" => "f1".to_string(),
        "F2" => "f2".to_string(),
        "F3" => "f3".to_string(),
        "F4" => "f4".to_string(),
        "F5" => "f5".to_string(),
        "F6" => "f6".to_string(),
        "F7" => "f7".to_string(),
        "F8" => "f8".to_string(),
        "F9" => "f9".to_string(),
        "F10" => "f10".to_string(),
        "F11" => "f11".to_string(),
        "F12" => "f12".to_string(),
        _ => {
            if key.len() == 1 {
                key.to_lowercase()
            } else {
                key
            }
        }
    }
}

impl HasWindowHandle for WebWindow {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let canvas_id = self.0.0.borrow().canvas_id;
        let handle = WebWindowHandle::new(canvas_id);
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::Web(handle)) })
    }
}

impl HasDisplayHandle for WebWindow {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        let handle = WebDisplayHandle::new();
        Ok(unsafe { DisplayHandle::borrow_raw(RawDisplayHandle::Web(handle)) })
    }
}

impl PlatformWindow for WebWindow {
    fn bounds(&self) -> Bounds<Pixels> {
        let state = self.0.0.borrow();
        Bounds {
            origin: Point::default(),
            size: state.size,
        }
    }

    fn is_maximized(&self) -> bool {
        false
    }

    fn window_bounds(&self) -> WindowBounds {
        WindowBounds::Windowed(self.bounds())
    }

    fn content_size(&self) -> Size<Pixels> {
        self.0.0.borrow().size
    }

    fn resize(&mut self, size: Size<Pixels>) {
        let mut state = self.0.0.borrow_mut();
        state.size = size;
        let scale_factor = state.scale_factor;

        if let Some(renderer) = &mut state.renderer {
            let device_size = Size {
                width: DevicePixels((size.width.0 * scale_factor) as i32),
                height: DevicePixels((size.height.0 * scale_factor) as i32),
            };
            renderer.update_drawable_size(device_size);
        }

        if let Some(canvas) = &state.canvas {
            let device_width = (size.width.0 * scale_factor) as u32;
            let device_height = (size.height.0 * scale_factor) as u32;
            canvas.set_width(device_width);
            canvas.set_height(device_height);

            let style = canvas.style();
            let _ = style.set_property("width", &format!("{}px", size.width.0));
            let _ = style.set_property("height", &format!("{}px", size.height.0));
        }
    }

    fn scale_factor(&self) -> f32 {
        self.0.0.borrow().scale_factor
    }

    fn appearance(&self) -> WindowAppearance {
        let prefers_dark = web_sys::window()
            .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok().flatten())
            .map(|mql| mql.matches())
            .unwrap_or(false);

        if prefers_dark {
            WindowAppearance::Dark
        } else {
            WindowAppearance::Light
        }
    }

    fn display(&self) -> Option<Rc<dyn PlatformDisplay>> {
        Some(Rc::new(WebDisplay::new()))
    }

    fn mouse_position(&self) -> Point<Pixels> {
        self.0.0.borrow().mouse_position
    }

    fn modifiers(&self) -> Modifiers {
        self.0.0.borrow().modifiers
    }

    fn capslock(&self) -> Capslock {
        self.0.0.borrow().capslock
    }

    fn set_input_handler(&mut self, input_handler: PlatformInputHandler) {
        let mut state = self.0.0.borrow_mut();
        state.input_handler = Some(input_handler);
        if let Some(ime_input) = &state.ime_input {
            let _ = ime_input.focus();
        }
    }

    fn take_input_handler(&mut self) -> Option<PlatformInputHandler> {
        self.0.0.borrow_mut().input_handler.take()
    }

    fn prompt(
        &self,
        _level: PromptLevel,
        msg: &str,
        _detail: Option<&str>,
        answers: &[PromptButton],
    ) -> Option<oneshot::Receiver<usize>> {
        let (tx, rx) = oneshot::channel();

        if let Some(window) = web_sys::window() {
            let confirmed = window.confirm_with_message(msg).unwrap_or(false);
            let answer_index = if confirmed { 0 } else { answers.len().saturating_sub(1) };
            let _ = tx.send(answer_index);
        }

        Some(rx)
    }

    fn activate(&self) {}

    fn is_active(&self) -> bool {
        web_sys::window()
            .and_then(|w| w.document())
            .map(|d| d.has_focus().unwrap_or(false))
            .unwrap_or(false)
    }

    fn is_hovered(&self) -> bool {
        self.0.0.borrow().is_hovered
    }

    fn set_title(&mut self, title: &str) {
        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            document.set_title(title);
        }
    }

    fn set_background_appearance(&self, _background_appearance: WindowBackgroundAppearance) {}

    fn minimize(&self) {}

    fn zoom(&self) {}

    fn toggle_fullscreen(&self) {
        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            if let Some(elem) = document.document_element() {
                let _ = elem.request_fullscreen();
            }
        }
    }

    fn is_fullscreen(&self) -> bool {
        web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.fullscreen_element())
            .is_some()
    }

    fn on_request_frame(&self, callback: Box<dyn FnMut(RequestFrameOptions)>) {
        self.0.0.borrow_mut().request_frame_callback = Some(callback);
    }

    fn on_input(&self, callback: Box<dyn FnMut(PlatformInput) -> DispatchEventResult>) {
        self.0.0.borrow_mut().input_callback = Some(callback);
    }

    fn on_active_status_change(&self, callback: Box<dyn FnMut(bool)>) {
        self.0.0.borrow_mut().active_callback = Some(callback);
    }

    fn on_hover_status_change(&self, callback: Box<dyn FnMut(bool)>) {
        self.0.0.borrow_mut().hover_callback = Some(callback);
    }

    fn on_resize(&self, callback: Box<dyn FnMut(Size<Pixels>, f32)>) {
        self.0.0.borrow_mut().resize_callback = Some(callback);
    }

    fn on_moved(&self, _callback: Box<dyn FnMut()>) {}

    fn on_should_close(&self, _callback: Box<dyn FnMut() -> bool>) {}

    fn on_hit_test_window_control(
        &self,
        _callback: Box<dyn FnMut() -> Option<crate::WindowControlArea>>,
    ) {
    }

    fn on_close(&self, callback: Box<dyn FnOnce()>) {
        self.0.0.borrow_mut().close_callback = Some(callback);
    }

    fn on_appearance_changed(&self, callback: Box<dyn FnMut()>) {
        self.0.0.borrow_mut().appearance_changed_callback = Some(callback);
    }

    fn draw(&self, scene: &Scene) {
        if let Some(renderer) = self.0.0.borrow_mut().renderer.as_mut() {
            renderer.draw(scene);
        }
    }

    fn sprite_atlas(&self) -> Arc<dyn PlatformAtlas> {
        self.0
            .0
            .borrow()
            .renderer
            .as_ref()
            .map(|r| r.sprite_atlas().clone() as Arc<dyn PlatformAtlas>)
            .unwrap_or_else(|| Arc::new(NoopAtlas))
    }

    fn gpu_specs(&self) -> Option<GpuSpecs> {
        self.0.0.borrow().renderer.as_ref().map(|r| r.gpu_specs())
    }

    fn is_renderer_ready(&self) -> bool {
        self.0.0.borrow().renderer.is_some()
    }

    fn update_ime_position(&self, bounds: Bounds<Pixels>) {
        let state = self.0.0.borrow();
        if let Some(ime_input) = &state.ime_input {
            let style = ime_input.style();
            let _ = style.set_property("left", &format!("{}px", bounds.origin.x.0));
            let _ = style.set_property("top", &format!("{}px", bounds.origin.y.0));
            let _ = style.set_property("width", &format!("{}px", bounds.size.width.0.max(1.0)));
            let _ = style.set_property("height", &format!("{}px", bounds.size.height.0.max(1.0)));
            let _ = ime_input.focus();
        }
    }
}

pub(crate) struct WebDisplay {
    bounds: Bounds<Pixels>,
}

impl WebDisplay {
    pub fn new() -> Self {
        let (width, height) = web_sys::window()
            .map(|w| {
                let width = w.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(1920.0);
                let height = w.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(1080.0);
                (width as f32, height as f32)
            })
            .unwrap_or((1920.0, 1080.0));

        Self {
            bounds: Bounds {
                origin: Point::default(),
                size: Size {
                    width: px(width),
                    height: px(height),
                },
            },
        }
    }
}

impl std::fmt::Debug for WebDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebDisplay")
            .field("bounds", &self.bounds)
            .finish()
    }
}

impl PlatformDisplay for WebDisplay {
    fn id(&self) -> crate::DisplayId {
        crate::DisplayId(0)
    }

    fn uuid(&self) -> Result<uuid::Uuid> {
        Ok(uuid::Uuid::nil())
    }

    fn bounds(&self) -> Bounds<Pixels> {
        self.bounds
    }
}

struct NoopAtlas;

impl PlatformAtlas for NoopAtlas {
    fn get_or_insert_with<'a>(
        &self,
        _key: &crate::AtlasKey,
        _build: &mut dyn FnMut() -> Result<Option<(Size<DevicePixels>, std::borrow::Cow<'a, [u8]>)>>,
    ) -> Result<Option<crate::AtlasTile>> {
        Ok(None)
    }

    fn remove(&self, _key: &crate::AtlasKey) {}
}

use wasm_bindgen::JsCast;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Bounds, PlatformWindow, WindowBounds, WindowParams, px};

    fn create_test_params() -> WindowParams {
        WindowParams {
            bounds: Bounds {
                origin: Point::default(),
                size: Size {
                    width: px(800.0),
                    height: px(600.0),
                },
            },
            ..Default::default()
        }
    }

    #[cfg(target_arch = "wasm32")]
    mod wasm_tests {
        use super::*;
        use wasm_bindgen_test::*;

        wasm_bindgen_test_configure!(run_in_browser);

        #[wasm_bindgen_test]
        fn test_window_creation() {
            let handle = crate::AnyWindowHandle::new(1, crate::AppEntityType::Window);
            let params = create_test_params();
            let window = WebWindow::new(handle, params);
            assert!(window.is_ok());
        }

        #[wasm_bindgen_test]
        fn test_window_canvas_id() {
            let handle = crate::AnyWindowHandle::new(1, crate::AppEntityType::Window);
            let params = create_test_params();
            let window = WebWindow::new(handle, params).unwrap();
            let id = window.canvas_id();
            assert!(id > 0);
        }

        #[wasm_bindgen_test]
        fn test_window_unique_canvas_ids() {
            let handle1 = crate::AnyWindowHandle::new(1, crate::AppEntityType::Window);
            let handle2 = crate::AnyWindowHandle::new(2, crate::AppEntityType::Window);
            let params = create_test_params();

            let window1 = WebWindow::new(handle1, params.clone()).unwrap();
            let window2 = WebWindow::new(handle2, params).unwrap();

            assert_ne!(window1.canvas_id(), window2.canvas_id());
        }

        #[wasm_bindgen_test]
        fn test_window_bounds() {
            let handle = crate::AnyWindowHandle::new(1, crate::AppEntityType::Window);
            let params = create_test_params();
            let window = WebWindow::new(handle, params).unwrap();

            let bounds = window.bounds();
            assert_eq!(bounds.size.width.0, 800.0);
            assert_eq!(bounds.size.height.0, 600.0);
        }

        #[wasm_bindgen_test]
        fn test_window_content_size() {
            let handle = crate::AnyWindowHandle::new(1, crate::AppEntityType::Window);
            let params = create_test_params();
            let window = WebWindow::new(handle, params).unwrap();

            let size = window.content_size();
            assert_eq!(size.width.0, 800.0);
            assert_eq!(size.height.0, 600.0);
        }

        #[wasm_bindgen_test]
        fn test_window_scale_factor() {
            let handle = crate::AnyWindowHandle::new(1, crate::AppEntityType::Window);
            let params = create_test_params();
            let window = WebWindow::new(handle, params).unwrap();

            let scale = window.scale_factor();
            assert!(scale > 0.0);
        }

        #[wasm_bindgen_test]
        fn test_window_is_not_maximized() {
            let handle = crate::AnyWindowHandle::new(1, crate::AppEntityType::Window);
            let params = create_test_params();
            let window = WebWindow::new(handle, params).unwrap();

            assert!(!window.is_maximized());
        }

        #[wasm_bindgen_test]
        fn test_window_bounds_type() {
            let handle = crate::AnyWindowHandle::new(1, crate::AppEntityType::Window);
            let params = create_test_params();
            let window = WebWindow::new(handle, params).unwrap();

            let window_bounds = window.window_bounds();
            assert!(matches!(window_bounds, WindowBounds::Windowed(_)));
        }

        #[wasm_bindgen_test]
        fn test_window_appearance() {
            let handle = crate::AnyWindowHandle::new(1, crate::AppEntityType::Window);
            let params = create_test_params();
            let window = WebWindow::new(handle, params).unwrap();

            let appearance = window.appearance();
            assert!(matches!(
                appearance,
                WindowAppearance::Light | WindowAppearance::Dark
            ));
        }

        #[wasm_bindgen_test]
        fn test_window_display() {
            let handle = crate::AnyWindowHandle::new(1, crate::AppEntityType::Window);
            let params = create_test_params();
            let window = WebWindow::new(handle, params).unwrap();

            let display = window.display();
            assert!(display.is_some());
        }

        #[wasm_bindgen_test]
        fn test_window_mouse_position() {
            let handle = crate::AnyWindowHandle::new(1, crate::AppEntityType::Window);
            let params = create_test_params();
            let window = WebWindow::new(handle, params).unwrap();

            let pos = window.mouse_position();
            assert_eq!(pos.x.0, 0.0);
            assert_eq!(pos.y.0, 0.0);
        }

        #[wasm_bindgen_test]
        fn test_window_modifiers() {
            let handle = crate::AnyWindowHandle::new(1, crate::AppEntityType::Window);
            let params = create_test_params();
            let window = WebWindow::new(handle, params).unwrap();

            let modifiers = window.modifiers();
            assert!(!modifiers.shift);
            assert!(!modifiers.control);
            assert!(!modifiers.alt);
            assert!(!modifiers.platform);
            assert!(!modifiers.function);
        }

        #[wasm_bindgen_test]
        fn test_window_renderer_not_ready_initially() {
            let handle = crate::AnyWindowHandle::new(1, crate::AppEntityType::Window);
            let params = create_test_params();
            let window = WebWindow::new(handle, params).unwrap();

            assert!(!window.is_renderer_ready());
        }

        #[wasm_bindgen_test]
        fn test_window_is_hovered_initially_false() {
            let handle = crate::AnyWindowHandle::new(1, crate::AppEntityType::Window);
            let params = create_test_params();
            let window = WebWindow::new(handle, params).unwrap();

            assert!(!window.is_hovered());
        }

        #[wasm_bindgen_test]
        fn test_window_resize() {
            let handle = crate::AnyWindowHandle::new(1, crate::AppEntityType::Window);
            let params = create_test_params();
            let mut window = WebWindow::new(handle, params).unwrap();

            let new_size = Size {
                width: px(1024.0),
                height: px(768.0),
            };
            window.resize(new_size);

            let size = window.content_size();
            assert_eq!(size.width.0, 1024.0);
            assert_eq!(size.height.0, 768.0);
        }
    }
}
