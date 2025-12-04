use std::time::Duration;

use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{HotKey, Modifiers},
};
use gpui::{App, AppContext, Application, AsyncApp};
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use objc2_foundation::MainThreadMarker;

use crate::panel::Panel;

mod input;
mod panel;
mod view;

fn main() {
    Application::new().run(|cx: &mut App| {
        // Set as accessory app (no dock icon) after gpui initializes
        if let Some(mtm) = MainThreadMarker::new() {
            let app = NSApplication::sharedApplication(mtm);
            app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
        }

        let panel = cx.new(|cx| Panel::new(cx));

        // global shortcut handling
        let hotkey_manager = Box::leak(Box::new(
            GlobalHotKeyManager::new().expect("failed to create global hotkey manager"),
        ));
        let hotkey = HotKey::new(Some(Modifiers::ALT), global_hotkey::hotkey::Code::Space);
        hotkey_manager
            .register(hotkey)
            .expect("failed to register global hotkey");
        let receiver = GlobalHotKeyEvent::receiver().clone();
        let panel_for_hotkey = panel.clone();
        cx.spawn({
            let panel = panel_for_hotkey;
            let receiver = receiver;
            move |cx: &mut AsyncApp| {
                let mut cx = cx.clone();
                async move {
                    loop {
                        while let Ok(event) = receiver.try_recv() {
                            if event.state == HotKeyState::Pressed {
                                let _ = panel.update(&mut cx, |panel, cx| panel.toggle(cx));
                            }
                        }

                        cx.background_executor()
                            .timer(Duration::from_millis(20))
                            .await;
                    }
                }
            }
        })
        .detach();
    });
}
