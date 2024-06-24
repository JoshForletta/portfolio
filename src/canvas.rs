#![allow(non_snake_case)]

use dioxus::prelude::*;
use futures_util::Future;
use wasm_bindgen::prelude::*;
use web_sys::{js_sys::Array, HtmlCanvasElement, ResizeObserver, ResizeObserverEntry};

#[derive(Clone)]
pub enum CanvasEvent<T> {
    Init(CanvasInitEvent<T>),
    Resized { width: u32, height: u32 },
    UserEvent(T),
}

impl<T> CanvasEvent<T> {
    pub fn init(self) -> Option<CanvasInitEvent<T>> {
        match self {
            Self::Init(init_event) => Some(init_event),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct CanvasInitEvent<T> {
    pub coroutine: Coroutine<CanvasEvent<T>>,
    pub mounted_event: Event<MountedData>,
    pub canvas: HtmlCanvasElement,
}

struct CanvasResizeEventDispatcher {
    _closure: Closure<dyn FnMut(Array)>,
    _observer: ResizeObserver,
}

impl CanvasResizeEventDispatcher {
    fn new<T: 'static>(
        canvas: &HtmlCanvasElement,
        canvas_event_tx: &Coroutine<CanvasEvent<T>>,
    ) -> Self {
        to_owned![canvas_event_tx];

        let closure = Closure::new(move |entries: Array| {
            let rect = entries
                .get(0)
                .unchecked_into::<ResizeObserverEntry>()
                .content_rect();

            let (width, height) = (rect.width() as u32, rect.height() as u32);

            canvas_event_tx.send(CanvasEvent::Resized { width, height });
        });

        let observer = ResizeObserver::new(closure.as_ref().unchecked_ref()).unwrap();
        observer.observe(canvas);

        Self {
            _closure: closure,
            _observer: observer,
        }
    }
}

pub fn use_canvas_coroutine<T: 'static, H, C, F>(
    cx: Scope,
    constructor: C,
) -> Coroutine<CanvasEvent<T>>
where
    H: FnMut(CanvasEvent<T>) + 'static,
    C: FnOnce(HtmlCanvasElement) -> F + 'static,
    F: Future<Output = Option<H>> + 'static,
{
    use_coroutine(cx, |mut rx: UnboundedReceiver<CanvasEvent<T>>| async move {
        use futures_util::StreamExt;

        let init_event = rx
            .next()
            .await
            .and_then(|event| event.init())
            .expect("init event");

        let _resize_event_dispatcher =
            CanvasResizeEventDispatcher::new(&init_event.canvas, &init_event.coroutine);

        let Some(mut event_handler) = constructor(init_event.canvas.clone()).await else {
            return;
        };

        event_handler(CanvasEvent::Init(init_event));

        while let Some(event) = rx.next().await {
            event_handler(event);
        }
    })
    .to_owned()
}

#[derive(Props)]
pub struct CanvasProps<'a> {
    id: &'a str,
    width: &'a str,
    height: &'a str,
}

pub fn Canvas<'a, T: Clone + 'static>(cx: Scope<'a, CanvasProps>) -> Element<'a> {
    let event_coroutine = use_coroutine_handle::<CanvasEvent<T>>(cx).unwrap();

    let CanvasProps {
        id, width, height, ..
    } = *cx.props;

    cx.render(rsx! {
        canvas {
            id: id,
            width: width,
            height: height,
            onmounted: move |mounted_event| {
                let canvas = web_sys::window()
                    .and_then(|window| window.document())
                    .and_then(|document| document.get_element_by_id(id))
                    .and_then(|element| element.dyn_into::<HtmlCanvasElement>().ok())
                    .expect("canvas element with `id` to be created");

                let init_event = CanvasEvent::Init(CanvasInitEvent {
                    coroutine: event_coroutine.to_owned(),
                    mounted_event,
                    canvas,
                });

                event_coroutine.send(init_event.clone());
            },

        }
    })
}
