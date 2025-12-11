use std::{sync::Arc, time::Duration};
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

use gpui::{
    Animation, AnimationExt, App, Application, Asset, AssetLogger, AssetSource, Bounds, Context,
    Hsla, ImageAssetLoader, ImageCacheError, ImgResourceLoader, LOADING_DELAY, Length, RenderImage,
    Resource, SharedString, Window, WindowBounds, WindowOptions, black, div, img, prelude::*,
    pulsating_between, px, red, size,
};

#[cfg(not(target_arch = "wasm32"))]
struct Assets {}

#[cfg(not(target_arch = "wasm32"))]
impl AssetSource for Assets {
    fn load(&self, path: &str) -> anyhow::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        std::fs::read(path)
            .map(Into::into)
            .map_err(Into::into)
            .map(Some)
    }

    fn list(&self, path: &str) -> anyhow::Result<Vec<SharedString>> {
        Ok(std::fs::read_dir(path)?
            .filter_map(|entry| {
                Some(SharedString::from(
                    entry.ok()?.path().to_string_lossy().into_owned(),
                ))
            })
            .collect::<Vec<_>>())
    }
}

#[cfg(not(target_arch = "wasm32"))]
const IMAGE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/image/app-icon.png");
#[cfg(target_arch = "wasm32")]
const IMAGE: &str = "assets/image/app-icon.png";

#[derive(Copy, Clone, Hash)]
struct LoadImageParameters {
    timeout: Duration,
    fail: bool,
}

struct LoadImageWithParameters {}

#[cfg(not(target_arch = "wasm32"))]
impl Asset for LoadImageWithParameters {
    type Source = LoadImageParameters;

    type Output = Result<Arc<RenderImage>, ImageCacheError>;

    fn load(
        parameters: Self::Source,
        cx: &mut App,
    ) -> impl std::future::Future<Output = Self::Output> + Send + 'static {
        let timer = cx.background_executor().timer(parameters.timeout);
        let data = AssetLogger::<ImageAssetLoader>::load(
            Resource::Path(Path::new(IMAGE).to_path_buf().into()),
            cx,
        );
        async move {
            timer.await;
            if parameters.fail {
                log::error!("Intentionally failed to load image");
                Err(anyhow::anyhow!("Failed to load image").into())
            } else {
                data.await
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl Asset for LoadImageWithParameters {
    type Source = LoadImageParameters;

    type Output = Result<Arc<RenderImage>, ImageCacheError>;

    fn load(
        parameters: Self::Source,
        cx: &mut App,
    ) -> impl std::future::Future<Output = Self::Output> + 'static {
        let timer = cx.background_executor().timer(parameters.timeout);
        let data = AssetLogger::<ImageAssetLoader>::load(Resource::Uri(IMAGE.into()), cx);
        async move {
            timer.await;
            if parameters.fail {
                log::error!("Intentionally failed to load image");
                Err(anyhow::anyhow!("Failed to load image").into())
            } else {
                data.await
            }
        }
    }
}

struct ImageLoadingExample {}

impl ImageLoadingExample {
    #[cfg(not(target_arch = "wasm32"))]
    fn nonexistent_image_source() -> Option<std::path::PathBuf> {
        Some(Path::new("this/file/really/shouldn't/exist/or/won't/be/an/image/I/hope").to_path_buf())
    }

    #[cfg(target_arch = "wasm32")]
    fn nonexistent_image_source() -> Option<std::path::PathBuf> {
        None
    }

    fn loading_element() -> impl IntoElement {
        div().size_full().flex_none().p_0p5().rounded_xs().child(
            div().size_full().with_animation(
                "loading-bg",
                Animation::new(Duration::from_secs(3))
                    .repeat()
                    .with_easing(pulsating_between(0.04, 0.24)),
                move |this, delta| this.bg(black().opacity(delta)),
            ),
        )
    }

    fn fallback_element() -> impl IntoElement {
        let fallback_color: Hsla = black().opacity(0.5);

        div().size_full().flex_none().p_0p5().child(
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .rounded_xs()
                .text_sm()
                .text_color(fallback_color)
                .border_1()
                .border_color(fallback_color)
                .child("?"),
        )
    }
}

impl Render for ImageLoadingExample {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().flex().flex_col().size_full().justify_around().child(
            div().flex().flex_row().w_full().justify_around().child(
                div()
                    .flex()
                    .bg(gpui::white())
                    .size(Length::Definite(px(300.0).into()))
                    .justify_center()
                    .items_center()
                    .child({
                        let image_source = LoadImageParameters {
                            timeout: LOADING_DELAY.saturating_sub(Duration::from_millis(25)),
                            fail: false,
                        };

                        // Load within the 'loading delay', should not show loading fallback
                        img(move |window: &mut Window, cx: &mut App| {
                            window.use_asset::<LoadImageWithParameters>(&image_source, cx)
                        })
                        .id("image-1")
                        .border_1()
                        .size_12()
                        .with_fallback(|| Self::fallback_element().into_any_element())
                        .border_color(red())
                        .with_loading(|| Self::loading_element().into_any_element())
                        .on_click(move |_, _, cx| {
                            cx.remove_asset::<LoadImageWithParameters>(&image_source);
                        })
                    })
                    .child({
                        // Load after a long delay
                        let image_source = LoadImageParameters {
                            timeout: Duration::from_secs(5),
                            fail: false,
                        };

                        img(move |window: &mut Window, cx: &mut App| {
                            window.use_asset::<LoadImageWithParameters>(&image_source, cx)
                        })
                        .id("image-2")
                        .with_fallback(|| Self::fallback_element().into_any_element())
                        .with_loading(|| Self::loading_element().into_any_element())
                        .size_12()
                        .border_1()
                        .border_color(red())
                        .on_click(move |_, _, cx| {
                            cx.remove_asset::<LoadImageWithParameters>(&image_source);
                        })
                    })
                    .child({
                        // Fail to load image after a long delay
                        let image_source = LoadImageParameters {
                            timeout: Duration::from_secs(5),
                            fail: true,
                        };

                        // Fail to load after a long delay
                        img(move |window: &mut Window, cx: &mut App| {
                            window.use_asset::<LoadImageWithParameters>(&image_source, cx)
                        })
                        .id("image-3")
                        .with_fallback(|| Self::fallback_element().into_any_element())
                        .with_loading(|| Self::loading_element().into_any_element())
                        .size_12()
                        .border_1()
                        .border_color(red())
                        .on_click(move |_, _, cx| {
                            cx.remove_asset::<LoadImageWithParameters>(&image_source);
                        })
                    })
                    .when_some(Self::nonexistent_image_source(), |this, source| {
                        // Ensure that the normal image loader doesn't spam logs
                        this.child(
                            img(source.clone())
                                .id("image-4")
                                .border_1()
                                .size_12()
                                .with_fallback(|| Self::fallback_element().into_any_element())
                                .border_color(red())
                                .with_loading(|| Self::loading_element().into_any_element())
                                .on_click(move |_, _, cx| {
                                    cx.remove_asset::<ImgResourceLoader>(&source.clone().into());
                                }),
                        )
                    }),
            ),
        )
    }
}

#[gpui::main]
fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();

    #[cfg(not(target_arch = "wasm32"))]
    let app = Application::new().with_assets(Assets {});
    #[cfg(target_arch = "wasm32")]
    let app = Application::new();

    app.run(|cx: &mut App| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(300.), px(300.)),
                cx,
            ))),
            ..Default::default()
        };
        cx.open_window(options, |_, cx| {
            cx.activate(false);
            cx.new(|_| ImageLoadingExample {})
        })
        .unwrap();
    });
}
