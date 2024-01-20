#![allow(non_snake_case)]

use dioxus::prelude::*;
use portfolio::canvas::{use_canvas_coroutine, Canvas, CanvasEvent};
use toodle::{
    math::Vector,
    render_layer::rectangle::{Rectangle, RectangleRendererConstructor},
    renderer::RendererBuilder,
};

#[derive(Debug, Clone, Copy)]
pub enum ExampleEvent {
    InsertRect(Rectangle),
}

fn main() {
    dioxus_logger::init(log::LevelFilter::Info).expect("failed to init logger");
    dioxus_web::launch(App);
}

fn App(cx: Scope) -> Element {
    let canvas_coroutine = use_canvas_coroutine(cx, |canvas| async move {
        let mut renderer_builder =
            RendererBuilder::new().dimensions(canvas.width(), canvas.height());

        renderer_builder.push_layer(RectangleRendererConstructor);

        let mut renderer = renderer_builder
            .build(wgpu::SurfaceTarget::Canvas(canvas))
            .await
            .ok()?;

        Some(move |event: CanvasEvent<ExampleEvent>| match event {
            CanvasEvent::Init { .. } => renderer.render().unwrap(),
            CanvasEvent::Resized { width, height } => renderer.resize(width, height),
            CanvasEvent::UserEvent(event) => match event {
                ExampleEvent::InsertRect(rect) => {
                    renderer.insert(rect);
                    renderer.render().unwrap();
                }
            },
        })
    });

    cx.render(rsx! {
        Canvas::<ExampleEvent> {
            id: "teeheee",
        }
        button {
            onclick: move |_| {
                canvas_coroutine.send(CanvasEvent::UserEvent(ExampleEvent::InsertRect(Rectangle {
                position: Vector([150.0, 75.0]),
                size: Vector([75.0, 48.5]),
                z_index: 0,
                corner_radii: Vector([48.4, 24.25, 12.125, 0.0]),
                color: Vector([0.8, 0.2, 0.1, 1.0]),
            })))},
            "add rectangle",
        }
    })
}
