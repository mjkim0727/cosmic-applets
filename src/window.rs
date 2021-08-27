use cascade::cascade;
use glib::clone;
use gtk4::{gdk, glib, prelude::*};
use std::cell::Cell;

use crate::status_area::StatusArea;
use crate::time_button::TimeButton;
use crate::x;

pub fn window(monitor: gdk::Monitor) -> gtk4::Window {
    let box_ = cascade! {
        gtk4::CenterBox::new();
        ..set_start_widget(Some(&cascade! {
            gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
            ..append(&gtk4::Button::with_label("Workspaces"));
            ..append(&gtk4::Button::with_label("Applications"));
        }));
        ..set_center_widget(Some(&TimeButton::new()));
        ..set_end_widget(Some(&StatusArea::new()));
    };

    let window = cascade! {
        gtk4::Window::new();
        ..set_decorated(false);
        ..set_child(Some(&box_));
        ..set_size_request(monitor.geometry().width, 0);
        ..connect_realize(|window| {
            let surface = window.surface().unwrap();
            let size = Cell::new(None);
            surface.connect_layout(move |surface, width, height| {
                if size.replace(Some((width, height))) != Some((width, height)) {
                    println!("height: {}", height);
                    println!("width: {}", width);
                }
            });
        });
        ..show();
    };

    fn monitor_geometry_changed(window: &gtk4::Window, monitor: &gdk::Monitor) {
        let geometry = monitor.geometry();
        window.set_size_request(geometry.width, 0);

        if let Some((display, surface)) = x::get_window_x11(&window) {
            let top: x::c_ulong = 32; // XXX arbitrary
            let top_start_x = geometry.x as x::c_ulong;
            let top_end_x = top_start_x + geometry.width as x::c_ulong - 1;

            unsafe {
                x::set_position(&display, &surface, top_start_x as _, 0);

                x::change_property(
                    &display,
                    &surface,
                    "_NET_WM_STRUT_PARTIAL",
                    x::PropMode::Replace,
                    &[0, 0, top, 0, 0, 0, 0, 0, top_start_x, top_end_x, 0, 0],
                );
            }
        }
    }

    if let Some((display, surface)) = x::get_window_x11(&window) {
        unsafe {
            surface.set_skip_pager_hint(true);
            surface.set_skip_taskbar_hint(true);
            x::wm_state_add(&display, &surface, "_NET_WM_STATE_ABOVE");
            x::wm_state_add(&display, &surface, "_NET_WM_STATE_STICKY");
            x::change_property(
                &display,
                &surface,
                "_NET_WM_ALLOWED_ACTIONS",
                x::PropMode::Replace,
                &[
                    x::Atom::new(&display, "_NET_WM_ACTION_CHANGE_DESKTOP").unwrap(),
                    x::Atom::new(&display, "_NET_WM_ACTION_ABOVE").unwrap(),
                    x::Atom::new(&display, "_NET_WM_ACTION_BELOW").unwrap(),
                ],
            );
            x::change_property(
                &display,
                &surface,
                "_NET_WM_WINDOW_TYPE",
                x::PropMode::Replace,
                &[x::Atom::new(&display, "_NET_WM_WINDOW_TYPE_DOCK").unwrap()],
            );
        }
    }

    monitor.connect_geometry_notify(clone!(@strong window => move |monitor| {
        monitor_geometry_changed(&window, &monitor);
    }));
    monitor_geometry_changed(&window, &monitor);

    monitor.connect_invalidate(clone!(@strong window => move |_| window.close()));

    window
}
