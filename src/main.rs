#![allow(non_snake_case)]

use std::time::Duration;

use dioxus::prelude::*;
use portfolio::canvas::{use_canvas_coroutine, Canvas, CanvasEvent};
use sdf::{
    rectangle::{Rectangle, RectangleRenderer},
    Device,
};
use wasmtimer::tokio::sleep;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RectangleEvent {
    SetPhase(f32),
}

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

        Some(move |event: CanvasEvent<RectangleEvent>| match event {
            CanvasEvent::Init { .. } => renderer.render().unwrap(),
            CanvasEvent::Resized { width, height } => renderer.resize(width, height),
            CanvasEvent::UserEvent(RectangleEvent::SetPhase(phase)) => {
                renderer.set_phase(phase);
                renderer.render().unwrap()
            }
        })
    });

    let tick = use_coroutine(cx, |mut exit: UnboundedReceiver<()>| async move {
        let mut phase = 0.0;

        loop {
            if exit.try_next().is_ok() {
                break;
            }

            sleep(Duration::from_millis(20)).await;

            phase += 0.1;
            canvas_coroutine.send(CanvasEvent::UserEvent(RectangleEvent::SetPhase(phase)));
        }
    });

    cx.render(rsx! {
        Canvas::<RectangleEvent> {
            id: "teeheee",
            width: "340",
            height: "280",
        }
        button {
            onclick: move |_| {
                tick.send(())
            },
            "stop",
        }
    })
}
