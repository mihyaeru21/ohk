use nwg::{
    Event, EventHandler, GlobalCursor, Icon, Menu, MenuItem, MessageWindow, NativeUi, NwgError,
    OemIcon, TrayNotification,
};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct SystemTray {
    window: MessageWindow,
    icon: Icon,
    tray: TrayNotification,
    menu: Menu,
    menu_item: MenuItem,
}

impl SystemTray {
    fn show_menu(&self) {
        let (x, y) = GlobalCursor::position();
        self.menu.popup(x, y);
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}

struct SystemTrayUi {
    inner: Rc<SystemTray>,
    default_handler: RefCell<Vec<EventHandler>>,
}

impl nwg::NativeUi<SystemTrayUi> for SystemTray {
    fn build_ui(mut data: Self) -> Result<SystemTrayUi, NwgError> {
        MessageWindow::builder().build(&mut data.window)?;

        Icon::builder()
            .source_system(Some(OemIcon::Sample)) // TODO: original icon
            .build(&mut data.icon)?;

        TrayNotification::builder()
            .parent(&data.window)
            .icon(Some(&data.icon))
            .build(&mut data.tray)?;

        Menu::builder()
            .popup(true)
            .parent(&data.window)
            .build(&mut data.menu)?;

        MenuItem::builder()
            .text("Exit")
            .parent(&data.menu)
            .build(&mut data.menu_item)?;

        let ui = SystemTrayUi {
            inner: Rc::new(data),
            default_handler: Default::default(),
        };

        let event_ui = Rc::downgrade(&ui.inner);
        let handle_events = move |event, _event_data, handle| {
            if let Some(event_ui) = event_ui.upgrade() {
                match event {
                    Event::OnContextMenu => {
                        if &handle == &event_ui.tray {
                            SystemTray::show_menu(&event_ui);
                        }
                    }
                    Event::OnMenuItemSelected => {
                        if &handle == &event_ui.menu_item {
                            SystemTray::exit(&event_ui);
                        }
                    }
                    _ => {}
                }
            }
        };

        ui.default_handler
            .borrow_mut()
            .push(nwg::full_bind_event_handler(
                &ui.inner.window.handle,
                handle_events,
            ));

        Ok(ui)
    }
}

impl Drop for SystemTrayUi {
    fn drop(&mut self) {
        let mut handlers = self.default_handler.borrow_mut();
        for handler in handlers.drain(0..) {
            nwg::unbind_event_handler(&handler);
        }
    }
}

pub fn run() {
    nwg::init().expect("oops!");
    let _ui = SystemTray::build_ui(Default::default()).expect("failed to build UI.");
    nwg::dispatch_thread_events();
}
