use std::time::Duration;

use gpui::{
    App, AppContext, AsyncApp, Context, Entity, FocusHandle, InteractiveElement, IntoElement,
    KeyDownEvent, KeystrokeEvent, ParentElement, Render, ScrollHandle, StatefulInteractiveElement,
    Styled, WeakEntity, Window, div, hsla,
};

use crate::input::TextInput;

pub struct View {
    cur_idx: usize,
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
    pub input: Entity<TextInput>,
}

impl View {
    pub fn new(cx: &mut App) -> Self {
        Self {
            cur_idx: 0,
            focus_handle: cx.focus_handle(),
            scroll_handle: ScrollHandle::new(),
            input: cx.new(|cx| {
                let input = TextInput::new(cx);
                cx.spawn(|this: WeakEntity<TextInput>, cx: &mut AsyncApp| {
                    let mut cx = cx.clone();
                    async move {
                        loop {
                            let epoch = this
                                .update(&mut cx, |input, _| input.blink_epoch)
                                .unwrap_or(0);
                            cx.background_executor()
                                .timer(Duration::from_millis(500))
                                .await;

                            match this.update(&mut cx, |input, cx| {
                                if epoch == input.blink_epoch {
                                    input.toggle_cursor();
                                    cx.notify();
                                }
                            }) {
                                Ok(_) => {}
                                Err(_) => break,
                            }
                        }
                    }
                })
                .detach();
                input
            }),
        }
    }

    pub fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for View {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                match event.keystroke.key.as_str() {
                    "escape" => {
                        cx.hide();
                        window.remove_window();
                    }
                    _ => {}
                }
            }))
            .h_full()
            .w_full()
            .text_color(hsla(0.0, 0.0, 0.9, 1.0))
            .bg(hsla(0.0, 0.0, 0.08, 0.5))
            .id("input")
            .track_focus(&self.focus_handle.clone())
            .overflow_x_scroll()
            .track_scroll(&self.scroll_handle.clone())
            .p_5()
            .child(self.input.clone())
    }
}
