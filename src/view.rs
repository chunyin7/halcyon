use std::{sync::mpsc::Sender, time::Duration};

use gpui::{
    App, AppContext, AsyncApp, Context, Entity, FocusHandle, Focusable, InteractiveElement,
    IntoElement, KeyDownEvent, ParentElement, Render, ScrollHandle, StatefulInteractiveElement,
    Styled, WeakEntity, Window, div, hsla,
};
use objc2::rc::Retained;
use objc2_foundation::{
    NSArray, NSMetadataItem, NSMetadataItemDisplayNameKey, NSMetadataItemPathKey, NSMetadataQuery,
    NSPredicate, NSString,
};

use crate::input::TextInput;

pub struct View {
    cur_idx: usize,
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
    pub input: Entity<TextInput>,
    query_tx: Sender<SearchQuery>,
}

pub struct SearchQuery {
    query: String,
    response_tx: oneshot::Sender<SearchResponse>,
}

struct CurrentQuery {
    q: Retained<NSMetadataQuery>,
    tx: oneshot::Sender<Vec<NSMetadataItem>>,
}

struct SearchItem;

pub struct SearchResponse;

impl View {
    pub fn new(cx: &mut App) -> Self {
        let (query_tx, query_rx) = std::sync::mpsc::channel::<SearchQuery>();

        cx.background_spawn(async move {
            let mut cur: Option<Retained<NSMetadataQuery>> = None;

            loop {
                if let Ok(query) = query_rx.try_recv() {
                    if let Some(cur) = cur.take() {
                        unsafe { cur.stopQuery() };
                    }

                    let SearchQuery { query , response_tx } = query;
                    let search_query = NSString::from_str(query.as_str());

                    cur = Some(unsafe { NSMetadataQuery::new() });
                    let format = NSString::from_str(
                        "kMDItemDisplayName LIKE %@, kMDItemContentType == \"com.apple.application\"",
                    );
                    let args = NSArray::new();
                    unsafe { args.arrayByAddingObject(search_query.as_ref()) };
                    let predicate = unsafe {
                        NSPredicate::predicateWithFormat_argumentArray(
                            format.as_ref(),
                            Some(args.as_ref()),
                        )
                    };
                    if let Some(query) = cur.as_mut() {
                        unsafe {
                            query.setPredicate(Some(predicate.as_ref()));
                            query.startQuery();
                        };
                    }
                }

                if let Some(cur) = cur.take() {
                    if !unsafe { cur.isGathering() } {
                        let results = unsafe { cur.results() };
                        let ret: Vec<SearchItem> = results.iter().map(|item| {
                            let item: &NSMetadataItem = item.downcast_ref().unwrap();
                            let path = unsafe { item.valueForAttribute(NSMetadataItemPathKey).unwrap() };
                            let name = unsafe { item.valueForAttribute(NSMetadataItemDisplayNameKey).unwrap() };

                            SearchItem
                        }).collect();
                    }
                }
            }
        })
        .detach();

        Self {
            cur_idx: 0,
            focus_handle: cx.focus_handle(),
            scroll_handle: ScrollHandle::new(),
            query_tx,
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
}

impl Focusable for View {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
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
