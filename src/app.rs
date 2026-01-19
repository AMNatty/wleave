use crate::button::{WButton, WButtonActionList, WButtonJustify};
use crate::config::AppConfig;
use crate::exec::run_command;
use crate::layout::MenuLayout;
use crate::paintable::svg_picture_colorized;
use glib::object::Cast;
use glib::timeout_add_local_once;
use glib_macros::clone;
use gtk4::prelude::{BoxExt, ButtonExt, GtkWindowExt, WidgetExt};
use gtk4::{EventControllerKey, GestureClick, PropagationPhase};
use gtk4_layer_shell::{KeyboardMode, LayerShell};
use std::sync::Arc;
use std::time::Duration;
use wleave::cli_opt::{AspectRatio, ButtonLayout, Protocol};

fn do_exit(window: &libadwaita::ApplicationWindow, _service_mode: bool) {
    window.close();
}

fn on_option(
    command_list: &WButtonActionList,
    delay_ms: u32,
    service_mode: bool,
    window: libadwaita::ApplicationWindow,
) {
    let Some(command) = command_list.enumerate().find(|w| w.is_applicable()) else {
        return;
    };

    let command = command.clone();

    window.connect_hide(clone!(
        #[strong]
        command,
        move |window| {
            timeout_add_local_once(
                Duration::from_millis(delay_ms.into()),
                clone!(
                    #[strong]
                    command,
                    #[weak_allow_none]
                    window,
                    move || {
                        run_command(command);
                        window.inspect(move |w| do_exit(w, service_mode));
                    }
                ),
            );
        }
    ));

    window.set_visible(false);
}

fn handle_key(
    config: &Arc<AppConfig>,
    window: &libadwaita::ApplicationWindow,
    key: &gtk4::gdk::Key,
) -> glib::Propagation {
    if let &gtk4::gdk::Key::Escape = key {
        do_exit(window, config.service);
        return glib::Propagation::Proceed;
    }

    let key = key
        .to_unicode()
        .map(|c| c.to_string())
        .or_else(|| key.name().map(|s| s.to_string()));

    if let Some(ref key_name) = key {
        let button = config.buttons.iter().find(|b| b.keybind == *key_name);

        if let Some(WButton { action, .. }) = button {
            on_option(
                action,
                config.delay_command_ms,
                config.service,
                window.clone(),
            );
        }
    }

    glib::Propagation::Proceed
}

pub fn create_app(
    config: &Arc<AppConfig>,
    app: &libadwaita::Application,
) -> libadwaita::ApplicationWindow {
    let service_mode = config.service;

    let container_box = gtk4::CenterBox::builder()
        .valign(gtk4::Align::Fill)
        .halign(gtk4::Align::Fill)
        .orientation(gtk4::Orientation::Vertical)
        .margin_top(config.margin_top.unwrap_or(config.margin))
        .margin_bottom(config.margin_bottom.unwrap_or(config.margin))
        .margin_start(config.margin_left.unwrap_or(config.margin))
        .margin_end(config.margin_right.unwrap_or(config.margin))
        .build();

    let window = libadwaita::ApplicationWindow::builder()
        .application(app)
        .title("wleave")
        .content(&container_box)
        .decorated(false)
        .build();

    match config.protocol {
        Protocol::LayerShell => {
            window.init_layer_shell();
            window.set_layer(gtk4_layer_shell::Layer::Overlay);
            window.set_namespace(Some("wleave"));
            window.set_exclusive_zone(-1);
            window.set_keyboard_mode(KeyboardMode::Exclusive);

            window.set_anchor(gtk4_layer_shell::Edge::Left, true);
            window.set_anchor(gtk4_layer_shell::Edge::Right, true);
            window.set_anchor(gtk4_layer_shell::Edge::Top, true);
            window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
        }
        Protocol::Xdg => {
            window.fullscreen();
        }
        Protocol::None => {}
    }

    if config.close_on_lost_focus {
        window.connect_is_active_notify(move |window| {
            if window.is_visible() && !window.is_active() && !service_mode {
                do_exit(window, service_mode);
            }
        });
    }

    let click_away_controller = GestureClick::builder()
        .propagation_phase(PropagationPhase::Bubble)
        .button(gtk4::gdk::BUTTON_PRIMARY)
        .n_points(1)
        .build();
    click_away_controller.connect_released(clone!(
        #[weak]
        window,
        #[upgrade_or_panic]
        move |_, _, _, _| {
            do_exit(&window, service_mode);
        }
    ));
    window.add_controller(click_away_controller);

    let key_controller = EventControllerKey::new();
    key_controller.connect_key_pressed(clone!(
        #[strong]
        config,
        #[weak]
        window,
        #[upgrade_or_panic]
        move |_, key, _, _| handle_key(&config, &window, &key)
    ));
    window.add_controller(key_controller);

    let buttons_container = gtk4::Box::builder()
        .valign(gtk4::Align::Fill)
        .halign(gtk4::Align::Fill)
        .layout_manager(&MenuLayout::new(
            config.button_aspect_ratio.map(AspectRatio::as_float),
            config.column_spacing,
            config.row_spacing,
        ))
        .build();

    let btn_count = config.buttons.len() as u32;
    let buttons_per_row = match config.buttons_per_row {
        ButtonLayout::PerRow(n) => n,
        ButtonLayout::RowRatio(n, d) => btn_count * n / d.min(btn_count * n),
    };

    for bttn in config.buttons.iter() {
        let justify = match bttn.justify {
            WButtonJustify::Center => gtk4::Justification::Center,
            WButtonJustify::Fill => gtk4::Justification::Fill,
            WButtonJustify::Left => gtk4::Justification::Left,
            WButtonJustify::Right => gtk4::Justification::Right,
        };

        let button = gtk4::Button::builder()
            .name(&bttn.label)
            .hexpand(true)
            .vexpand(true)
            .cursor(&gdk4::Cursor::from_name("pointer", None).expect("pointer cursor not found"))
            .build();

        let overlay = gtk4::Overlay::builder().vexpand(true).hexpand(true).build();

        if config.show_keybinds {
            let key_label = gtk4::Label::builder()
                .label(format!("[{}]", bttn.keybind))
                .halign(gtk4::Align::Start)
                .valign(gtk4::Align::Start)
                .css_classes(["dimmed", "keybind"])
                .build();

            overlay.add_overlay(&key_label);
        }

        let inner = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .valign(gtk4::Align::Center)
            .build();

        let picture = if let Some(icon) = &bttn.icon {
            let picture = if icon.ends_with(".svg") {
                svg_picture_colorized(icon).upcast()
            } else {
                gtk4::Picture::for_filename(icon)
            };

            picture.set_content_fit(gtk4::ContentFit::ScaleDown);
            picture.add_css_class("icon-dropshadow");

            inner.append(&picture);
            Some(picture)
        } else {
            None
        };

        let label = gtk4::Label::builder()
            .label(&bttn.text)
            .css_classes(["action-name"])
            .use_markup(true)
            .justify(justify)
            .build();

        // Picture being none means the old system to configure buttons is used
        if bttn.width.is_some() || bttn.height.is_some() || picture.is_none() {
            label.set_xalign(bttn.width.unwrap_or(0.5));
            label.set_yalign(bttn.height.unwrap_or(0.9));
            overlay.add_overlay(&label);
        } else {
            inner.insert_child_after(&label, picture.as_ref());
        }

        overlay.set_child(Some(&inner));

        if bttn.circular {
            button.add_css_class("circular");
        }

        button.connect_clicked(clone!(
            #[weak]
            window,
            #[to_owned(rename_to = action)]
            &bttn.action,
            #[to_owned(rename_to = delay_ms)]
            &config.delay_command_ms,
            #[upgrade_or_panic]
            move |_| on_option(&action, delay_ms, service_mode, window)
        ));

        button.set_child(Some(&overlay));

        buttons_container.append(&button);
    }

    container_box.set_shrink_center_last(false);
    container_box.set_center_widget(Some(&buttons_container));

    if !config.no_version_info {
        let version_info = gtk4::Label::builder()
        .label(format!(
            "Wleave {}. <a href=\"https://github.com/AMNatty/wleave/releases/tag/0.6.0\">Missing or broken icons?</a>",
            env!("CARGO_PKG_VERSION")
        ))
        .use_markup(true)
        .can_focus(false)
        .css_classes(["dimmed", "version-info"])
        .margin_top(12)
        .build();
        container_box.set_end_widget(Some(&version_info));
    }

    window
}
