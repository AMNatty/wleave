use clap::Parser;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use gtk::gdk::{keys, EventKey, Screen};
use gtk::glib::Propagation;
use gtk::prelude::*;
use gtk::{gio, Application, ApplicationWindow, CssProvider, Label, StyleContext};
use gtk_layer_shell::LayerShell;
use serde::Deserialize;
use wleave::cli_opt::{Args, Protocol};

#[derive(Debug)]
struct WButtonConfig {
    buttons: Vec<WButton>,
}

#[derive(Debug, Deserialize)]
struct WButton {
    label: String,
    action: String,
    text: String,
    keybind: String,
    #[serde(default = "default_justify")]
    justify: String,
    #[serde(default = "default_width")]
    width: f32,
    #[serde(default = "default_height")]
    height: f32,
    #[serde(default = "default_circular")]
    circular: bool,
}

fn default_justify() -> String {
    String::from("center")
}

fn default_width() -> f32 {
    0.5
}

fn default_height() -> f32 {
    0.9
}

fn default_circular() -> bool {
    false
}

#[derive(Debug)]
struct AppConfig {
    margin_left: i32,
    margin_right: i32,
    margin_top: i32,
    margin_bottom: i32,
    column_spacing: u32,
    row_spacing: u32,
    protocol: Protocol,
    buttons_per_row: u32,
    close_on_lost_focus: bool,
    button_config: WButtonConfig,
    show_keybinds: bool,
}

fn load_file_search<S>(
    given_file: Option<&impl AsRef<Path>>,
    file_name: &impl AsRef<Path>,
    load_func: impl Fn(&dyn AsRef<Path>) -> Result<Option<S>, String>,
) -> Result<S, String> {
    if let Some(given_file) = given_file {
        return match load_func(&given_file) {
            Ok(Some(config)) => Ok(config),
            Ok(None) => Err(format!(
                "Failed to load {}: File does not exist",
                given_file.as_ref().display()
            )),
            Err(e) => Err(e),
        };
    }

    let user_config_dir = dirs::config_dir().unwrap_or_else(|| {
        dirs::home_dir().map_or_else(|| Path::new("~/.config").to_owned(), |p| p.join(".config"))
    });

    let user_css_path = user_config_dir.join("wleave");
    let user_css_path_compat = user_config_dir.join("wlogout");

    for path in &[
        user_css_path.as_ref(),
        user_css_path_compat.as_ref(),
        Path::new("/etc/wleave"),
        Path::new("/etc/wlogout"),
        Path::new("/usr/local/etc/wleave"),
        Path::new("/usr/local/etc/wlogout"),
    ] {
        let full_path = path.join(file_name);
        if let Some(config) = load_func(&full_path)? {
            eprintln!("File found in: {}", full_path.display());
            return Ok(config);
        } else {
            eprintln!("No file found in: {}", full_path.display());
        }
    }

    Err(format!("No {} file found!", file_name.as_ref().display()))
}

fn load_config_from_file(path: &dyn AsRef<Path>) -> Result<Option<WButtonConfig>, String> {
    if !path.as_ref().is_file() {
        return Ok(None);
    }

    let file = std::fs::File::open(path)
        .map_err(|e| format!("Failed to open file {}: {e}", path.as_ref().display()))?;

    let reader = std::io::BufReader::new(file);

    let mut buttons = Vec::new();

    let mut de = serde_json::Deserializer::from_reader(reader);

    loop {
        match WButton::deserialize(&mut de) {
            Ok(button) => buttons.push(button),
            Err(e) if e.is_eof() => break Ok(Some(WButtonConfig { buttons })),
            Err(e) => break Err(format!("Parsing failed: {e}")),
        }
    }
}

fn load_config(file: Option<&impl AsRef<Path>>) -> Result<WButtonConfig, String> {
    load_file_search(file, &"layout", load_config_from_file)
}

fn load_css_from_file(path: &dyn AsRef<Path>) -> Result<Option<CssProvider>, String> {
    if !path.as_ref().is_file() {
        return Ok(None);
    }

    let provider = CssProvider::new();
    provider
        .load_from_file(&gio::File::for_path(path))
        .map_err(|e| format!("Failed to load CSS: {e}"))?;
    Ok(Some(provider))
}

fn load_css(file: Option<&impl AsRef<Path>>) -> Result<CssProvider, String> {
    load_file_search(file, &"style.css", load_css_from_file)
}

fn run_command(command: &str) {
    if let Err(e) = Command::new("sh").args(["-c", command]).spawn() {
        eprintln!("Execution error: {e}");
    }
}

fn handle_key(config: &Arc<AppConfig>, window: &ApplicationWindow, e: &EventKey) -> Propagation {
    match e.keyval() {
        keys::constants::Escape => {
            window.close();
        }
        other => {
            let key = other
                .to_unicode()
                .map(|c| c.to_string())
                .or_else(|| other.name().map(|s| s.to_string()));

            if let Some(ref key_name) = key {
                let button = config
                    .button_config
                    .buttons
                    .iter()
                    .find(|b| b.keybind == *key_name);

                if let Some(WButton { action, .. }) = button {
                    run_command(action);
                    window.close();
                }
            }
        }
    }

    Propagation::Proceed
}

fn app_main(config: &Arc<AppConfig>, app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("wleave")
        .build();

    match config.protocol {
        Protocol::LayerShell => {
            window.init_layer_shell();
            window.set_layer(gtk_layer_shell::Layer::Overlay);
            window.set_namespace("wleave");
            window.set_exclusive_zone(-1);
            window.set_keyboard_interactivity(true);

            window.set_anchor(gtk_layer_shell::Edge::Left, true);
            window.set_anchor(gtk_layer_shell::Edge::Right, true);
            window.set_anchor(gtk_layer_shell::Edge::Top, true);
            window.set_anchor(gtk_layer_shell::Edge::Bottom, true);
        }
        Protocol::Xdg => {
            window.fullscreen();
        }
    }

    if config.close_on_lost_focus {
        window.connect_focus_out_event(|window, _| {
            window.close();
            Propagation::Proceed
        });
    }

    let cfg = config.clone();
    window.connect_key_press_event(move |window, e| handle_key(&cfg, window, e));

    let grid = gtk::Grid::new();

    window.add(&grid);

    grid.set_column_spacing(config.column_spacing);
    grid.set_row_spacing(config.row_spacing);
    grid.set_margin_top(config.margin_top);
    grid.set_margin_bottom(config.margin_bottom);
    grid.set_margin_start(config.margin_left);
    grid.set_margin_end(config.margin_right);

    for (i, bttn) in config.button_config.buttons.iter().enumerate() {
        let label = if config.show_keybinds {
            format!("{} [{}]", bttn.text, bttn.keybind)
        } else {
            bttn.text.to_owned()
        };

        let justify = match bttn.justify.as_str() {
            "center" => gtk::Justification::Center,
            "fill" => gtk::Justification::Fill,
            "left" => gtk::Justification::Left,
            "right" => gtk::Justification::Right,
            _ => gtk::Justification::Center
        };

        let button = gtk::Button::builder()
            .label(&label)
            .name(&bttn.label)
            .hexpand(true)
            .vexpand(true)
            .build();

        if let Some(label) = button.child() {
            if let Some(label) = label.downcast_ref::<Label>() {
                label.set_xalign(bttn.width);
                label.set_yalign(bttn.height);
                label.set_use_markup(true);
                label.set_justify(justify);
            }
        }

        if bttn.circular {
            button.style_context().add_class("circular");
        }

        let action = bttn.action.clone();
        button.connect_clicked(move |_| run_command(&action));

        let x = i as u32 % config.buttons_per_row;
        let y = i as u32 / config.buttons_per_row;

        grid.attach(&button, x as i32, y as i32, 1, 1);
    }

    window.show_all();
}

fn main() {
    let args = Args::parse();

    let button_config = match load_config(args.layout.as_ref()) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load config: {e}");
            return;
        }
    };

    let config = Arc::new(AppConfig {
        margin_top: args.margin_top.unwrap_or(args.margin),
        margin_bottom: args.margin_bottom.unwrap_or(args.margin),
        margin_left: args.margin_left.unwrap_or(args.margin),
        margin_right: args.margin_right.unwrap_or(args.margin),
        row_spacing: args.row_spacing,
        column_spacing: args.column_spacing,
        protocol: args.protocol,
        buttons_per_row: args.buttons_per_row,
        close_on_lost_focus: args.close_on_lost_focus,
        show_keybinds: args.show_keybinds,
        button_config,
    });

    let app = Application::builder()
        .application_id("sh.natty.Wleave")
        .build();

    app.connect_startup(move |_| match load_css(args.css.as_ref()) {
        Ok(css) => StyleContext::add_provider_for_screen(
            &Screen::default().expect("Could not connect to a display."),
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        ),
        Err(e) => eprintln!("Failed to load CSS: {e}"),
    });

    app.connect_activate(move |app| app_main(&config, app));

    app.run_with_args(&[] as &[&str]);
}
