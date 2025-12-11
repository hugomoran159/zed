use gpui::{
    App, Application, Background, Bounds, ColorSpace, Context, Path, PathBuilder, Pixels,
    Window, WindowBounds, WindowOptions, canvas, div,
    linear_color_stop, linear_gradient, point, prelude::*, px, quad, rgb, size,
};
use wasm_bindgen::prelude::wasm_bindgen;

struct PaintingDemo {
    paths: Vec<(Path<Pixels>, Background)>,
    background_quads: Vec<(Bounds<Pixels>, Background)>,
}

impl PaintingDemo {
    fn new() -> Self {
        let mut paths = vec![];

        // Black squares beneath transparent paths
        let background_quads = vec![
            (
                Bounds {
                    origin: point(px(70.), px(70.)),
                    size: size(px(40.), px(40.)),
                },
                gpui::black().into(),
            ),
            (
                Bounds {
                    origin: point(px(170.), px(70.)),
                    size: size(px(40.), px(40.)),
                },
                gpui::black().into(),
            ),
            (
                Bounds {
                    origin: point(px(270.), px(70.)),
                    size: size(px(40.), px(40.)),
                },
                gpui::black().into(),
            ),
            (
                Bounds {
                    origin: point(px(370.), px(70.)),
                    size: size(px(40.), px(40.)),
                },
                gpui::black().into(),
            ),
            (
                Bounds {
                    origin: point(px(450.), px(50.)),
                    size: size(px(80.), px(80.)),
                },
                gpui::black().into(),
            ),
        ];

        // 50% opaque red path that extends across black quad
        let mut builder = PathBuilder::fill();
        builder.move_to(point(px(50.), px(50.)));
        builder.line_to(point(px(130.), px(50.)));
        builder.line_to(point(px(130.), px(130.)));
        builder.line_to(point(px(50.), px(130.)));
        builder.close();
        let path = builder.build().unwrap();
        let mut red = rgb(0xFF0000);
        red.a = 0.5;
        paths.push((path, red.into()));

        // 50% opaque blue path that extends across black quad
        let mut builder = PathBuilder::fill();
        builder.move_to(point(px(150.), px(50.)));
        builder.line_to(point(px(230.), px(50.)));
        builder.line_to(point(px(230.), px(130.)));
        builder.line_to(point(px(150.), px(130.)));
        builder.close();
        let path = builder.build().unwrap();
        let mut blue = rgb(0x0000FF);
        blue.a = 0.5;
        paths.push((path, blue.into()));

        // 50% opaque green path that extends across black quad
        let mut builder = PathBuilder::fill();
        builder.move_to(point(px(250.), px(50.)));
        builder.line_to(point(px(330.), px(50.)));
        builder.line_to(point(px(330.), px(130.)));
        builder.line_to(point(px(250.), px(130.)));
        builder.close();
        let path = builder.build().unwrap();
        let mut green = rgb(0x00FF00);
        green.a = 0.5;
        paths.push((path, green.into()));

        // 50% opaque black path that extends across black quad
        let mut builder = PathBuilder::fill();
        builder.move_to(point(px(350.), px(50.)));
        builder.line_to(point(px(430.), px(50.)));
        builder.line_to(point(px(430.), px(130.)));
        builder.line_to(point(px(350.), px(130.)));
        builder.close();
        let path = builder.build().unwrap();
        let mut black = rgb(0x000000);
        black.a = 0.5;
        paths.push((path, black.into()));

        // Two 50% opaque red circles overlapping - center should be darker red
        let mut builder = PathBuilder::fill();
        let center = point(px(530.), px(85.));
        let radius = px(30.);
        builder.move_to(point(center.x + radius, center.y));
        builder.arc_to(
            point(radius, radius),
            px(0.),
            false,
            false,
            point(center.x - radius, center.y),
        );
        builder.arc_to(
            point(radius, radius),
            px(0.),
            false,
            false,
            point(center.x + radius, center.y),
        );
        builder.close();
        let path = builder.build().unwrap();
        let mut red1 = rgb(0xFF0000);
        red1.a = 0.5;
        paths.push((path, red1.into()));

        let mut builder = PathBuilder::fill();
        let center = point(px(570.), px(85.));
        let radius = px(30.);
        builder.move_to(point(center.x + radius, center.y));
        builder.arc_to(
            point(radius, radius),
            px(0.),
            false,
            false,
            point(center.x - radius, center.y),
        );
        builder.arc_to(
            point(radius, radius),
            px(0.),
            false,
            false,
            point(center.x + radius, center.y),
        );
        builder.close();
        let path = builder.build().unwrap();
        let mut red2 = rgb(0xFF0000);
        red2.a = 0.5;
        paths.push((path, red2.into()));

        // draw a Rust logo
        let mut builder = lyon::path::Path::svg_builder();
        lyon::extra::rust_logo::build_logo_path(&mut builder);
        let mut builder: PathBuilder = builder.into();
        builder.translate(point(px(10.), px(200.)));
        builder.scale(0.9);
        let path = builder.build().unwrap();
        paths.push((path, gpui::black().into()));

        // draw a lightening bolt ⚡
        let mut builder = PathBuilder::fill();
        builder.add_polygon(
            &[
                point(px(150.), px(300.)),
                point(px(200.), px(225.)),
                point(px(200.), px(275.)),
                point(px(250.), px(200.)),
            ],
            false,
        );
        let path = builder.build().unwrap();
        paths.push((path, rgb(0x1d4ed8).into()));

        // draw a ⭐
        let mut builder = PathBuilder::fill();
        builder.move_to(point(px(350.), px(200.)));
        builder.line_to(point(px(370.), px(260.)));
        builder.line_to(point(px(430.), px(260.)));
        builder.line_to(point(px(380.), px(300.)));
        builder.line_to(point(px(400.), px(360.)));
        builder.line_to(point(px(350.), px(320.)));
        builder.line_to(point(px(300.), px(360.)));
        builder.line_to(point(px(320.), px(300.)));
        builder.line_to(point(px(270.), px(260.)));
        builder.line_to(point(px(330.), px(260.)));
        builder.line_to(point(px(350.), px(200.)));
        let path = builder.build().unwrap();
        paths.push((
            path,
            linear_gradient(
                180.,
                linear_color_stop(rgb(0xFACC15), 0.7),
                linear_color_stop(rgb(0xD56D0C), 1.),
            )
            .color_space(ColorSpace::Oklab),
        ));

        // draw linear gradient curve
        let square_bounds = Bounds {
            origin: point(px(450.), px(200.)),
            size: size(px(200.), px(80.)),
        };
        let height = square_bounds.size.height;
        let horizontal_offset = height;
        let vertical_offset = px(30.);
        let mut builder = PathBuilder::fill();
        builder.move_to(square_bounds.bottom_left());
        builder.curve_to(
            square_bounds.origin + point(horizontal_offset, vertical_offset),
            square_bounds.origin + point(px(0.0), vertical_offset),
        );
        builder.line_to(square_bounds.top_right() + point(-horizontal_offset, vertical_offset));
        builder.curve_to(
            square_bounds.bottom_right(),
            square_bounds.top_right() + point(px(0.0), vertical_offset),
        );
        builder.line_to(square_bounds.bottom_left());
        let path = builder.build().unwrap();
        paths.push((
            path,
            linear_gradient(
                180.,
                linear_color_stop(gpui::blue(), 0.4),
                linear_color_stop(gpui::red(), 1.),
            ),
        ));

        // draw a pie chart
        let center = point(px(96.), px(96.));
        let pie_center = point(px(775.), px(255.));
        let segments = [
            (
                point(px(871.), px(255.)),
                point(px(747.), px(163.)),
                rgb(0x1374e9),
            ),
            (
                point(px(747.), px(163.)),
                point(px(679.), px(263.)),
                rgb(0xe13527),
            ),
            (
                point(px(679.), px(263.)),
                point(px(754.), px(349.)),
                rgb(0x0751ce),
            ),
            (
                point(px(754.), px(349.)),
                point(px(854.), px(310.)),
                rgb(0x209742),
            ),
            (
                point(px(854.), px(310.)),
                point(px(871.), px(255.)),
                rgb(0xfbc10a),
            ),
        ];

        for (start, end, color) in segments {
            let mut builder = PathBuilder::fill();
            builder.move_to(start);
            builder.arc_to(center, px(0.), false, false, end);
            builder.line_to(pie_center);
            builder.close();
            let path = builder.build().unwrap();
            paths.push((path, color.into()));
        }

        Self {
            paths,
            background_quads,
        }
    }
}

impl Render for PaintingDemo {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let paths = self.paths.clone();
        let background_quads = self.background_quads.clone();

        div()
            .bg(gpui::white())
            .size_full()
            .p_4()
            .flex()
            .flex_col()
            .child(
                div()
                    .text_lg()
                    .text_color(gpui::black())
                    .child("GPUI Web - Painting Example (PathBuilder paths)")
            )
            .child(
                canvas(
                    move |_, _, _| {},
                    move |_, _, window, _| {
                        // First draw background quads
                        for (bounds, color) in background_quads.iter() {
                            window.paint_quad(quad(
                                *bounds,
                                px(0.),
                                *color,
                                px(0.),
                                gpui::transparent_black(),
                                Default::default(),
                            ));
                        }

                        // Then draw the paths on top
                        for (path, color) in paths.iter() {
                            window.paint_path(path.clone(), *color);
                        }
                    },
                )
                .size_full(),
            )
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    web_sys::console::log_1(&"Starting GPUI web painting example...".into());

    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(900.), px(500.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| PaintingDemo::new())
            },
        )
        .unwrap();
    });
}
