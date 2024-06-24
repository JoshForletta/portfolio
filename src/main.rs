#![allow(non_snake_case)]

use dioxus::prelude::*;
use portfolio::canvas::{use_canvas_coroutine, Canvas, CanvasEvent};
use sdf::{
    rectangle::{Rectangle, RectangleRenderer},
    Device,
};

fn main() {
    dioxus_logger::init(log::LevelFilter::Info).expect("failed to init logger");
    dioxus_web::launch(App);
}

fn App(cx: Scope) -> Element {
    let canvas_coroutine = use_canvas_coroutine(cx, |canvas| async move {
        let (width, height) = (canvas.width(), canvas.height());

        let rectangle = Rectangle {
            position: (width as f32 / 2.0, height as f32 / 2.0).into(),
            half_dimensions: (width as f32 / 4.0, height as f32 / 4.0).into(),
            corner_radii: (0.0, 0.0, 0.0, 0.0).into(),
            inner_color: (0.2, 0.4, 0.8, 1.0).into(),
            outer_color: (0.8, 0.4, 0.2, 1.0).into(),
            phase: 0.0,
            _padding: [0; 3],
        };
        let mut renderer = RectangleRenderer::new(
            Device::new(width, height, wgpu::SurfaceTarget::Canvas(canvas))
                .await
                .unwrap(),
            rectangle,
        );

        Some(move |event: CanvasEvent<()>| match event {
            CanvasEvent::Init { .. } => renderer.render().unwrap(),
            CanvasEvent::Resized { width, height } => renderer.resize(width, height),
            CanvasEvent::UserEvent(_) => (),
        })
    });

    cx.render(rsx! {
        Canvas::<()> {
            id: "teeheee",
            width: "340",
            height: "280",
        }
        // button {
        //     onclick: move |_| {
        //         canvas_coroutine.send(CanvasEvent::UserEvent(ExampleEvent::InsertRect(Rectangle {
        //         position: Vector([150.0, 75.0]),
        //         size: Vector([75.0, 48.5]),
        //         z_index: 0,
        //         corner_radii: Vector([48.4, 24.25, 12.125, 0.0]),
        //         color: Vector([0.8, 0.2, 0.1, 1.0]),
        //     })))},
        //     "add rectangle",
        // }
    })
}
