mod button;
mod config;
mod paintable;

use clap::Parser;
use glib::clone;
use miette::{Diagnostic, NamedSource, SourceOffset};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use tracing::{Level, error};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::button::WButton;
use crate::config::{AppConfig, load_config, load_css, merge_with_args};
use crate::paintable::svg_picture_colorized;
use gtk4::gdk::{Cursor, Display};
use gtk4::glib::{Propagation, timeout_add_local_once};
use gtk4::{ApplicationWindow, GestureClick, PropagationPhase};
use gtk4::{EventControllerKey, prelude::*};
use gtk4_layer_shell::{KeyboardMode, LayerShell};
use thiserror::Error;
use wleave::cli_opt::{Args, ButtonLayout, Protocol};

#[derive(Error, Diagnostic, Debug)]
pub enum WError {
    #[error("Failed to load the specified configuration file {0} as it does not exist")]
    SpecifiedPathNotAFile(PathBuf),
    #[error("Failed to find the configuration file {0} in the search path")]
    FileNotInSearchPath(PathBuf),
    #[error("An error has occurred while reading file {0}: {1}")]
    IoError(PathBuf, std::io::Error),
    #[error("JSON parsing failed")]
    #[diagnostic(code(wleave::parse_failed))]
    FileParseFailed(
        #[source_code] NamedSource<String>,
        #[label("The parser failed here")] SourceOffset,
        #[source] serde_json::Error,
    ),
    #[error("Failed to load CSS from file {0}: {1}")]
    CssReadError(PathBuf, glib::Error),
}

fn run_command(command: &str) {
    if let Err(e) = Command::new("sh").args(["-c", command]).spawn() {
        error!("Execution error: {e}");
    }
}

fn on_option(command: &str, delay_ms: u32, window: ApplicationWindow) {
    window.connect_hide(clone!(
        #[to_owned]
        command,
        #[weak]
        window,
        #[upgrade_or_panic]
        move |_| {
            timeout_add_local_once(
                Duration::from_millis(delay_ms.into()),
                clone!(
                    #[to_owned]
                    command,
                    #[weak_allow_none]
                    window,
                    move || {
                        run_command(&command);
                        window.inspect(ApplicationWindow::close);
                    }
                ),
            );
        }
    ));

    window.set_visible(false);
}

fn handle_key(
    config: &Arc<AppConfig>,
    window: &ApplicationWindow,
    key: &gtk4::gdk::Key,
) -> Propagation {
    if let &gtk4::gdk::Key::Escape = key {
        window.close();
        return Propagation::Proceed;
    }

    let key = key
        .to_unicode()
        .map(|c| c.to_string())
        .or_else(|| key.name().map(|s| s.to_string()));

    if let Some(ref key_name) = key {
        let button = config.buttons.iter().find(|b| b.keybind == *key_name);

        if let Some(WButton { action, .. }) = button {
            let state_action = action.clone();
            on_option(&state_action, config.delay_command_ms, window.clone());
        }
    }

    Propagation::Proceed
}

fn app_main(config: &Arc<AppConfig>, app: &libadwaita::Application) {
    let container_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_top(config.margin_top.unwrap_or(config.margin))
        .margin_bottom(config.margin_bottom.unwrap_or(config.margin))
        .margin_start(config.margin_left.unwrap_or(config.margin))
        .margin_end(config.margin_right.unwrap_or(config.margin))
        .build();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("wleave")
        .child(&container_box)
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
    }

    if config.close_on_lost_focus {
        window.connect_is_active_notify(|window| {
            if window.is_visible() && !window.is_active() {
                window.close();
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
        move |_, _, _, _| window.close()
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

    let grid = gtk4::Grid::new();
    grid.set_column_spacing(config.column_spacing);
    grid.set_row_spacing(config.row_spacing);

    let btn_count = config.buttons.len() as u32;
    let buttons_per_row = match config.buttons_per_row {
        ButtonLayout::PerRow(n) => n,
        ButtonLayout::RowRatio(n, d) => btn_count * n / d.min(btn_count * n),
    };

    for (i, bttn) in config.buttons.iter().enumerate() {
        let justify = match bttn.justify.as_str() {
            "center" => gtk4::Justification::Center,
            "fill" => gtk4::Justification::Fill,
            "left" => gtk4::Justification::Left,
            "right" => gtk4::Justification::Right,
            _ => gtk4::Justification::Center,
        };

        let button = gtk4::Button::builder()
            .name(&bttn.label)
            .hexpand(true)
            .vexpand(true)
            .cursor(&Cursor::from_name("pointer", None).expect("pointer cursor not found"))
            .build();

        let overlay = gtk4::Overlay::new();

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

            inner.insert_child_after(&picture, Option::<&gtk4::Widget>::None);
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

        button.set_child(Some(&overlay));

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
            move |_| on_option(&action, delay_ms, window)
        ));

        let x = i as u32 % buttons_per_row;
        let y = i as u32 / buttons_per_row;

        grid.attach(&button, x as i32, y as i32, 1, 1);
    }

    container_box.insert_child_after(&grid, Option::<&gtk4::Widget>::None);

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
        container_box.insert_child_after(&version_info, Some(&grid));
    }

    window.present();
}

fn on_startup(config: &AppConfig) {
    let display = Display::default().expect("Could not connect to a display");

    match load_css(config.css.as_ref()) {
        Ok(css) => gtk4::style_context_add_provider_for_display(
            &display,
            &css,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        ),
        Err(e) => error!("Failed to load CSS: {e}"),
    };
}

fn main() -> miette::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().without_time())
        .with(
            EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let args = Args::parse();

    let mut config = load_config(args.layout.as_ref())?;
    merge_with_args(&mut config, &args);

    let config = Arc::new(config);

    let app = libadwaita::Application::builder()
        .application_id("sh.natty.Wleave")
        .build();

    app.connect_startup(clone!(
        #[strong]
        config,
        move |_| on_startup(config.as_ref())
    ));

    app.connect_activate(move |app| app_main(&config, app));

    app.run_with_args(&[] as &[&str]);

    Ok(())
}
