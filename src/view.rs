use gpui::{
    App, AppContext, Context, Entity, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    Render, ScrollHandle, Styled, Window, div, hsla,
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
            input: cx.new(|cx| TextInput::new(cx)),
        }
    }

    pub fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for View {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h_full()
            .w_full()
            .text_color(hsla(0.0, 0.0, 0.9, 1.0))
            .bg(hsla(0.0, 0.0, 0.08, 0.5))
            .id("input")
            .track_focus(&self.focus_handle)
            .p_5()
            .child(self.input.clone())
    }
}
