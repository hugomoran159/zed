use crate::{
    Action, AnyWindowHandle, BackgroundExecutor, ClipboardItem, CursorStyle, ForegroundExecutor,
    Keymap, Menu, MenuItem, PathPromptOptions, Platform, PlatformDisplay, PlatformTextSystem,
    PlatformKeyboardLayout, PlatformKeyboardMapper, Task, WindowAppearance, WindowParams,
};
use anyhow::Result;
use collections::HashMap;
use futures::channel::oneshot;
use parking_lot::Mutex;
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use super::{WebDispatcher, WebDisplay, WebTextSystem, WebWindow};

pub(crate) struct WebPlatform {
    dispatcher: Arc<WebDispatcher>,
    background_executor: BackgroundExecutor,
    foreground_executor: ForegroundExecutor,
    text_system: Arc<WebTextSystem>,
    clipboard: Mutex<Option<ClipboardItem>>,
    quit_callbacks: RefCell<Vec<Box<dyn FnMut()>>>,
    windows: RefCell<HashMap<AnyWindowHandle, WebWindow>>,
}

impl WebPlatform {
    pub fn new() -> Rc<Self> {
        let dispatcher = Arc::new(WebDispatcher::new());
        let background_executor = BackgroundExecutor::new(dispatcher.clone());
        let foreground_executor = ForegroundExecutor::new(dispatcher.clone());
        let text_system = Arc::new(WebTextSystem::new());

        Rc::new(Self {
            dispatcher,
            background_executor,
            foreground_executor,
            text_system,
            clipboard: Mutex::new(None),
            quit_callbacks: RefCell::new(Vec::new()),
            windows: RefCell::new(HashMap::default()),
        })
    }
}

impl Platform for WebPlatform {
    fn background_executor(&self) -> BackgroundExecutor {
        self.background_executor.clone()
    }

    fn foreground_executor(&self) -> ForegroundExecutor {
        self.foreground_executor.clone()
    }

    fn text_system(&self) -> Arc<dyn PlatformTextSystem> {
        self.text_system.clone()
    }

    fn run(&self, on_finish_launching: Box<dyn FnOnce()>) {
        on_finish_launching();
    }

    fn quit(&self) {
        for callback in self.quit_callbacks.borrow_mut().iter_mut() {
            callback();
        }
    }

    fn restart(&self, _binary_path: Option<PathBuf>) {
        if let Some(window) = web_sys::window() {
            let _ = window.location().reload();
        }
    }

    fn activate(&self, _ignoring_other_apps: bool) {}

    fn hide(&self) {}

    fn hide_other_apps(&self) {}

    fn unhide_other_apps(&self) {}

    fn displays(&self) -> Vec<Rc<dyn PlatformDisplay>> {
        vec![Rc::new(WebDisplay::new())]
    }

    fn primary_display(&self) -> Option<Rc<dyn PlatformDisplay>> {
        Some(Rc::new(WebDisplay::new()))
    }

    fn active_window(&self) -> Option<AnyWindowHandle> {
        self.windows.borrow().keys().next().cloned()
    }

    fn open_window(
        &self,
        handle: AnyWindowHandle,
        params: WindowParams,
    ) -> Result<Box<dyn crate::PlatformWindow>> {
        let window = WebWindow::new(handle, params)?;
        self.windows.borrow_mut().insert(handle, window.clone());

        // Spawn async renderer initialization
        let window_clone = window.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = window_clone.initialize_renderer().await {
                web_sys::console::error_1(&format!("Failed to initialize renderer: {:?}", e).into());
            } else {
                web_sys::console::log_1(&"WebGPU renderer initialized".into());
            }
        });

        Ok(Box::new(window))
    }

    fn window_appearance(&self) -> WindowAppearance {
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

    fn open_url(&self, url: &str) {
        if let Some(window) = web_sys::window() {
            let _ = window.open_with_url_and_target(url, "_blank");
        }
    }

    fn on_open_urls(&self, _callback: Box<dyn FnMut(Vec<String>)>) {}

    fn register_url_scheme(&self, _url: &str) -> Task<Result<()>> {
        Task::ready(Ok(()))
    }

    fn prompt_for_paths(
        &self,
        _options: PathPromptOptions,
    ) -> oneshot::Receiver<Result<Option<Vec<PathBuf>>>> {
        let (tx, rx) = oneshot::channel();
        let _ = tx.send(Ok(None));
        rx
    }

    fn prompt_for_new_path(
        &self,
        _directory: &Path,
        _suggested_name: Option<&str>,
    ) -> oneshot::Receiver<Result<Option<PathBuf>>> {
        let (tx, rx) = oneshot::channel();
        let _ = tx.send(Ok(None));
        rx
    }

    fn can_select_mixed_files_and_dirs(&self) -> bool {
        false
    }

    fn reveal_path(&self, _path: &Path) {}

    fn open_with_system(&self, _path: &Path) {}

    fn on_quit(&self, callback: Box<dyn FnMut()>) {
        self.quit_callbacks.borrow_mut().push(callback);
    }

    fn on_reopen(&self, _callback: Box<dyn FnMut()>) {}

    fn set_menus(&self, _menus: Vec<Menu>, _keymap: &Keymap) {}

    fn set_dock_menu(&self, _menu: Vec<MenuItem>, _keymap: &Keymap) {}

    fn on_app_menu_action(&self, _callback: Box<dyn FnMut(&dyn Action)>) {}

    fn on_will_open_app_menu(&self, _callback: Box<dyn FnMut()>) {}

    fn on_validate_app_menu_command(&self, _callback: Box<dyn FnMut(&dyn Action) -> bool>) {}

    fn app_path(&self) -> Result<PathBuf> {
        Ok(PathBuf::from("/"))
    }

    fn path_for_auxiliary_executable(&self, _name: &str) -> Result<PathBuf> {
        Ok(PathBuf::from("/"))
    }

    fn set_cursor_style(&self, style: CursorStyle) {
        let cursor = match style {
            CursorStyle::Arrow => "default",
            CursorStyle::IBeam => "text",
            CursorStyle::Crosshair => "crosshair",
            CursorStyle::ClosedHand => "grabbing",
            CursorStyle::OpenHand => "grab",
            CursorStyle::PointingHand => "pointer",
            CursorStyle::ResizeLeft => "w-resize",
            CursorStyle::ResizeRight => "e-resize",
            CursorStyle::ResizeLeftRight => "ew-resize",
            CursorStyle::ResizeUp => "n-resize",
            CursorStyle::ResizeDown => "s-resize",
            CursorStyle::ResizeUpDown => "ns-resize",
            CursorStyle::ResizeColumn => "col-resize",
            CursorStyle::ResizeRow => "row-resize",
            CursorStyle::ResizeUpLeftDownRight => "nwse-resize",
            CursorStyle::ResizeUpRightDownLeft => "nesw-resize",
            CursorStyle::IBeamCursorForVerticalLayout => "vertical-text",
            CursorStyle::OperationNotAllowed => "not-allowed",
            CursorStyle::DragLink => "alias",
            CursorStyle::DragCopy => "copy",
            CursorStyle::ContextualMenu => "context-menu",
            CursorStyle::None => "none",
        };

        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            if let Some(body) = document.body() {
                let _ = body.style().set_property("cursor", cursor);
            }
        }
    }

    fn should_auto_hide_scrollbars(&self) -> bool {
        false
    }

    fn write_to_clipboard(&self, item: ClipboardItem) {
        *self.clipboard.lock() = Some(item.clone());

        if let Some(window) = web_sys::window() {
            let clipboard = window.navigator().clipboard();
            if let Some(text) = item.text() {
                let _ = clipboard.write_text(&text);
            }
        }
    }

    fn read_from_clipboard(&self) -> Option<ClipboardItem> {
        self.clipboard.lock().clone()
    }

    fn write_credentials(&self, _url: &str, _username: &str, _password: &[u8]) -> Task<Result<()>> {
        Task::ready(Ok(()))
    }

    fn read_credentials(&self, _url: &str) -> Task<Result<Option<(String, Vec<u8>)>>> {
        Task::ready(Ok(None))
    }

    fn delete_credentials(&self, _url: &str) -> Task<Result<()>> {
        Task::ready(Ok(()))
    }

    fn keyboard_layout(&self) -> Box<dyn PlatformKeyboardLayout> {
        Box::new(WebKeyboardLayout::default())
    }

    fn keyboard_mapper(&self) -> Rc<dyn PlatformKeyboardMapper> {
        Rc::new(WebKeyboardMapper)
    }

    fn on_keyboard_layout_change(&self, _callback: Box<dyn FnMut()>) {}
}

pub(crate) struct WebKeyboardLayout {
    id: String,
    name: String,
}

impl Default for WebKeyboardLayout {
    fn default() -> Self {
        let language = web_sys::window()
            .and_then(|w| w.navigator().language())
            .unwrap_or_else(|| "en-US".to_string());
        Self {
            id: language.clone(),
            name: language,
        }
    }
}

impl PlatformKeyboardLayout for WebKeyboardLayout {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }
}

pub(crate) struct WebKeyboardMapper;

impl PlatformKeyboardMapper for WebKeyboardMapper {
    fn map_key_equivalent(
        &self,
        keystroke: crate::Keystroke,
        _use_key_equivalents: bool,
    ) -> crate::KeybindingKeystroke {
        crate::KeybindingKeystroke::from_keystroke(keystroke)
    }

    fn get_key_equivalents(&self) -> Option<&collections::HashMap<char, char>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ClipboardItem, CursorStyle, Platform, PlatformKeyboardLayout};

    #[cfg(target_arch = "wasm32")]
    mod wasm_tests {
        use super::*;
        use wasm_bindgen_test::*;

        wasm_bindgen_test_configure!(run_in_browser);

        #[wasm_bindgen_test]
        fn test_platform_creation() {
            let platform = WebPlatform::new();
            assert!(platform.displays().len() >= 1);
        }

        #[wasm_bindgen_test]
        fn test_platform_primary_display() {
            let platform = WebPlatform::new();
            let display = platform.primary_display();
            assert!(display.is_some());
        }

        #[wasm_bindgen_test]
        fn test_clipboard_write_read() {
            let platform = WebPlatform::new();
            assert_eq!(platform.read_from_clipboard(), None);

            let item = ClipboardItem::new_string("test".to_string());
            platform.write_to_clipboard(item.clone());
            assert_eq!(platform.read_from_clipboard(), Some(item));
        }

        #[wasm_bindgen_test]
        fn test_clipboard_overwrite() {
            let platform = WebPlatform::new();

            let item1 = ClipboardItem::new_string("first".to_string());
            platform.write_to_clipboard(item1);

            let item2 = ClipboardItem::new_string("second".to_string());
            platform.write_to_clipboard(item2.clone());

            assert_eq!(platform.read_from_clipboard(), Some(item2));
        }

        #[wasm_bindgen_test]
        fn test_keyboard_layout() {
            let platform = WebPlatform::new();
            let layout = platform.keyboard_layout();
            assert!(!layout.id().is_empty());
            assert!(!layout.name().is_empty());
        }

        #[wasm_bindgen_test]
        fn test_keyboard_mapper() {
            let platform = WebPlatform::new();
            let mapper = platform.keyboard_mapper();
            assert!(mapper.get_key_equivalents().is_none());
        }

        #[wasm_bindgen_test]
        fn test_app_path() {
            let platform = WebPlatform::new();
            let path = platform.app_path();
            assert!(path.is_ok());
        }

        #[wasm_bindgen_test]
        fn test_cursor_style_mapping() {
            let platform = WebPlatform::new();
            platform.set_cursor_style(CursorStyle::Arrow);
            platform.set_cursor_style(CursorStyle::IBeam);
            platform.set_cursor_style(CursorStyle::PointingHand);
            platform.set_cursor_style(CursorStyle::ResizeLeftRight);
        }

        #[wasm_bindgen_test]
        fn test_window_appearance() {
            let platform = WebPlatform::new();
            let appearance = platform.window_appearance();
            assert!(matches!(
                appearance,
                crate::WindowAppearance::Light | crate::WindowAppearance::Dark
            ));
        }

        #[wasm_bindgen_test]
        fn test_can_select_mixed_files_and_dirs() {
            let platform = WebPlatform::new();
            assert!(!platform.can_select_mixed_files_and_dirs());
        }

        #[wasm_bindgen_test]
        fn test_should_auto_hide_scrollbars() {
            let platform = WebPlatform::new();
            assert!(!platform.should_auto_hide_scrollbars());
        }
    }
}
