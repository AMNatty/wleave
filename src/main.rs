use clap::{ArgAction, Parser, ValueEnum};
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use gtk::gdk::{keys, EventKey, Screen};
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, CssProvider, Label, StyleContext};
use serde::Deserialize;

#[derive(Debug, Copy, Clone, ValueEnum)]
enum Protocol {
    LayerShell,
    Xdg,
}

#[derive(Parser, Debug)]
#[command(author, version, disable_version_flag = true, about, long_about = None)]
struct Args {
    #[arg(short = 'v', long, action = ArgAction::Version)]
    version: Option<bool>,

    /// Specify a layout file
    #[arg(short = 'l', long)]
    layout: Option<String>,

    /// Specify a custom CSS file
    #[arg(short = 'C', long)]
    css: Option<String>,

    /// Set the number of buttons per row
    #[arg(short = 'b', long = "buttons-per-row", default_value_t = 3)]
    buttons_per_row: u32,

    /// Set space between buttons columns
    #[arg(short = 'c', long = "column-spacing", default_value_t = 5)]
    column_spacing: u32,

    /// Set space between buttons rows
    #[arg(short = 'r', long = "row-spacing", default_value_t = 5)]
    row_spacing: u32,

    /// Set the margin around buttons
    #[arg(short = 'm', long, default_value_t = 230)]
    margin: i32,

    /// Set margin for the left of buttons
    #[arg(short = 'L', long)]
    margin_left: Option<i32>,

    /// Set margin for the right of buttons
    #[arg(short = 'R', long)]
    margin_right: Option<i32>,

    /// Set margin for the top of buttons
    #[arg(short = 'T', long)]
    margin_top: Option<i32>,

    /// Set the margin for the bottom of buttons
    #[arg(short = 'B', long)]
    margin_bottom: Option<i32>,

    /// Close the menu on lost focus
    #[arg(short = 'f', long)]
    close_on_lost_focus: bool,

    /// Use layer-shell or xdg protocol
    #[arg(short = 'p', long, value_enum, default_value_t = Protocol::Xdg)]
    protocol: Protocol,
}

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
    #[serde(default = "default_width")]
    width: f32,
    #[serde(default = "default_height")]
    height: f32,
    #[serde(default = "default_circular")]
    circular: bool,
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
}

fn load_file_search<'b, S>(
    given_file: Option<&'b str>,
    file_name: &impl AsRef<Path>,
    load_func: impl Fn(&(dyn AsRef<Path> + 'b)) -> Result<Option<S>, String>,
) -> Result<S, String> {
    if let Some(given_file) = given_file {
        return match load_func(&given_file) {
            Ok(Some(config)) => Ok(config),
            Ok(None) => Err(format!("Failed to load {given_file}: File does not exist")),
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

fn load_config_from_file(
    path: &(impl AsRef<Path> + ?Sized),
) -> Result<Option<WButtonConfig>, String> {
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

fn load_config(file: Option<&str>) -> Result<WButtonConfig, String> {
    load_file_search(file, &"layout", load_config_from_file)
}

fn load_css_from_file(path: &(impl AsRef<Path> + ?Sized)) -> Result<Option<CssProvider>, String> {
    if !path.as_ref().is_file() {
        return Ok(None);
    }

    let css_data =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read CSS file: {e}"))?;

    let provider = CssProvider::new();
    provider
        .load_from_data(css_data.as_ref())
        .map_err(|e| format!("Failed to load CSS: {e}"))?;
    Ok(Some(provider))
}

fn load_css(file: Option<&str>) -> Result<CssProvider, String> {
    load_file_search(file, &"style.css", load_css_from_file)
}

fn run_command(command: &str) {
    if let Err(e) = Command::new("sh").args(["-c", command]).spawn() {
        eprintln!("Execution error: {e}");
    }
}

fn handle_key(config: &Arc<AppConfig>, window: &ApplicationWindow, e: &EventKey) -> Inhibit {
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

    Inhibit(false)
}

fn app_main(config: &Arc<AppConfig>, app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("wleave")
        .build();

    match config.protocol {
        Protocol::LayerShell => {
            gtk_layer_shell::init_for_window(&window);
            gtk_layer_shell::set_layer(&window, gtk_layer_shell::Layer::Overlay);
            gtk_layer_shell::set_exclusive_zone(&window, -1);
            gtk_layer_shell::set_keyboard_interactivity(&window, true);

            gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Left, true);
            gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Right, true);
            gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Top, true);
            gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Bottom, true);
        }
        Protocol::Xdg => {
            window.fullscreen();
        }
    }

    if config.close_on_lost_focus {
        window.connect_focus_out_event(|window, _| {
            window.close();
            Inhibit(false)
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
        let button = gtk::Button::builder()
            .label(&bttn.text)
            .name(&bttn.label)
            .hexpand(true)
            .vexpand(true)
            .build();

        if let Some(label) = button.child() {
            if let Some(label) = label.downcast_ref::<Label>() {
                label.set_xalign(bttn.width);
                label.set_yalign(bttn.height);
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

    let button_config = match load_config(args.layout.as_ref().map(String::as_ref)) {
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
        button_config,
    });

    let app = Application::builder()
        .application_id("sh.natty.Wleave")
        .build();

    app.connect_startup(
        move |_| match load_css(args.css.as_ref().map(String::as_ref)) {
            Ok(css) => StyleContext::add_provider_for_screen(
                &Screen::default().expect("Could not connect to a display."),
                &css,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            ),
            Err(e) => eprintln!("Failed to load CSS: {e}"),
        },
    );

    app.connect_activate(move |app| app_main(&config, app));

    app.run_with_args(&[] as &[&str]);
}
