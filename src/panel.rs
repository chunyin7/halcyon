use dispatch2::run_on_main;
use gpui::{
    App, AppContext, Bounds, Focusable, WindowBackgroundAppearance, WindowBounds, WindowHandle,
    WindowKind, WindowOptions, point, px, size,
};
use objc2_app_kit::NSEvent;

use crate::view::View;

pub struct Panel {
    window: WindowHandle<View>,
}

impl Panel {
    const WIDTH: f32 = 650.0;
    const HEIGHT: f32 = 75.0;

    pub fn new(cx: &mut App) -> Self {
        let window = Self::open_window(cx);
        Self { window }
    }

    fn open_window(cx: &mut App) -> WindowHandle<View> {
        let mouse_pos = run_on_main(|_mtm| unsafe { NSEvent::mouseLocation() });

        let displays = cx.displays();
        let active = displays.iter().find(move |display| {
            let bounds = display.bounds();
            mouse_pos.x >= bounds.origin.x.to_f64()
                && mouse_pos.x <= (bounds.origin.x + bounds.size.width).to_f64()
                && mouse_pos.y >= bounds.origin.y.to_f64()
                && mouse_pos.y <= (bounds.origin.y + bounds.size.height).to_f64()
        });

        let bounds = if let Some(display) = active {
            // appkit gives relative to bottom of screen, gpui expects relative to top of screen
            let bounds = display.bounds();
            Bounds::new(
                point(
                    bounds.center().x - px(Self::WIDTH / 2.0),
                    bounds.size.height * 0.2,
                ),
                size(px(Self::WIDTH), px(Self::HEIGHT)),
            )
        } else {
            Bounds::centered(None, size(px(Self::WIDTH), px(Self::HEIGHT)), cx)
        };

        let window = cx
            .open_window(
                WindowOptions {
                    titlebar: None,
                    is_movable: false,
                    kind: WindowKind::PopUp,
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_background: WindowBackgroundAppearance::Blurred,
                    display_id: if let Some(display) = active {
                        Some(display.id())
                    } else {
                        None
                    },
                    ..Default::default()
                },
                move |_window, cx| {
                    cx.new(|cx| {
                        let view = View::new(cx);
                        view
                    })
                },
            )
            .unwrap();

        window
            .update(cx, |view, window, cx| {
                window.focus(&view.input.focus_handle(cx));
            })
            .unwrap();

        window
    }

    pub fn hide(&mut self, cx: &mut App) {
        let _ = self.window.update(cx, |_view, window, cx| {
            cx.hide();
            window.remove_window();
        });
    }

    pub fn show(&mut self, cx: &mut App) {
        *self = Self::new(cx);
    }

    pub fn toggle(&mut self, cx: &mut App) {
        if let Some(_) = self.window.is_active(cx) {
            self.hide(cx);
        } else {
            self.show(cx);
        }
    }
}
