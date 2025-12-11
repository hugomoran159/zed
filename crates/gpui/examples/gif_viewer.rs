use gpui::{
    App, Application, Context, ImageSource, Render, SharedUri, Window, WindowOptions,
    div, img, prelude::*,
};
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

struct GifViewer {
    gif_source: ImageSource,
}

impl GifViewer {
    fn new(gif_source: ImageSource) -> Self {
        Self { gif_source }
    }
}

impl Render for GifViewer {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(
            img(self.gif_source.clone())
                .size_full()
                .object_fit(gpui::ObjectFit::Contain)
                .id("gif"),
        )
    }
}

#[gpui::main]
fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();
    Application::new().run(|cx: &mut App| {
        #[cfg(not(target_arch = "wasm32"))]
        let gif_source: ImageSource =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/image/black-cat-typing.gif").into();
        #[cfg(target_arch = "wasm32")]
        let gif_source: ImageSource = ImageSource::from(SharedUri::from("assets/image/black-cat-typing.gif"));

        cx.open_window(
            WindowOptions {
                focus: true,
                ..Default::default()
            },
            |_, cx| cx.new(|_| GifViewer::new(gif_source)),
        )
        .unwrap();
        cx.activate(true);
    });
}
