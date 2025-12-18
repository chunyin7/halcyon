use std::{path::PathBuf, sync::mpsc::Sender, time::Duration};

use gpui::{
    App, AppContext, AsyncApp, Context, Entity, FocusHandle, Focusable, InteractiveElement,
    IntoElement, KeyDownEvent, ParentElement, Render, ScrollHandle, Size,
    StatefulInteractiveElement, Styled, WeakEntity, Window, div, hsla, px,
};
use objc2::rc::Retained;
use objc2_foundation::{
    NSArray, NSMetadataItem, NSMetadataItemDisplayNameKey, NSMetadataItemPathKey, NSMetadataQuery,
    NSPredicate, NSString,
};

use crate::{input::TextInput, panel::Panel};

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
    tx: oneshot::Sender<SearchResponse>,
}

struct SearchItem {
    name: String,
    path: PathBuf,
}

pub struct SearchResponse {
    results: Vec<SearchItem>,
}

impl View {
    pub fn is_input_empty(&self, cx: &App) -> bool {
        self.input.read(cx).content().is_empty()
    }

    pub fn new(cx: &mut App, window: &Window) -> Self {
        let (query_tx, query_rx) = std::sync::mpsc::channel::<SearchQuery>();

        cx.background_spawn(async move {
            let mut cur: Option<CurrentQuery> = None;

            loop {
                if let Ok(query) = query_rx.try_recv() {
                    if let Some(cur) = cur.take() {
                        unsafe { cur.q.stopQuery() };
                    }

                    let SearchQuery { query , response_tx } = query;
                    let search_query = NSString::from_str(query.as_str());

                    cur = Some(CurrentQuery {
                        q: unsafe { NSMetadataQuery::new() },
                        tx: response_tx,
                    });
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
                            query.q.setPredicate(Some(predicate.as_ref()));
                            query.q.startQuery();
                        };
                    }
                }

                if let Some(cur) = cur.take() {
                    let CurrentQuery { q, tx } = cur;
                    if !unsafe { q.isGathering() } {
                        let results = unsafe { q.results() };
                        let results: Vec<SearchItem> = results.iter().map(|item| {
                            let item: &NSMetadataItem = item.downcast_ref().unwrap();
                            let path = unsafe {
                                item.valueForAttribute(NSMetadataItemPathKey).unwrap()
                            }
                            .downcast_ref::<NSString>()
                            .unwrap()
                            .to_string();
                            let path = PathBuf::from(path);
                            let name = unsafe {
                                item.valueForAttribute(NSMetadataItemDisplayNameKey).unwrap()
                            }
                            .downcast_ref::<NSString>()
                            .unwrap()
                            .to_string();

                            SearchItem {
                                path,
                                name,
                            }
                        }).collect();

                        tx.send(SearchResponse { results }).unwrap();
                    }
                }
            }
        })
        .detach();

        let input = cx.new(|cx| {
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
        });

        let window_handle = window.window_handle();
        cx.observe(&input, move |input, cx| {
            if !input.read(cx).content().is_empty() {
                let _ = window_handle.update(cx, |_view, window, _cx| {
                    window.resize(Size::new(px(Panel::WIDTH), px(Panel::EXPANDED_HEIGHT)));
                });
            } else {
                let _ = window_handle.update(cx, |_view, window, _cx| {
                    window.resize(Size::new(px(Panel::WIDTH), px(Panel::HEIGHT)));
                });
            }
        })
        .detach();

        Self {
            cur_idx: 0,
            focus_handle: cx.focus_handle(),
            scroll_handle: ScrollHandle::new(),
            query_tx,
            input,
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
