use clap::Parser;
use glib::clone;
use miette::{Diagnostic, NamedSource, Report, SourceOffset};
use std::borrow::Cow;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, enabled, error, info, warn, Level};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use gtk4::gdk::{Cursor, Display, Key};
use gtk4::glib::{timeout_add_local_once, Propagation};
use gtk4::{
    gio, Application, ApplicationWindow, CssProvider, GestureClick, Label, PropagationPhase,
};
use gtk4::{prelude::*, EventControllerKey};
use gtk4_layer_shell::{KeyboardMode, LayerShell};
use serde::Deserialize;
use thiserror::Error;
use wleave::cli_opt::{Args, Protocol};

#[derive(Debug, Deserialize)]
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
    delay_ms: u32,
    protocol: Protocol,
    buttons_per_row: u32,
    close_on_lost_focus: bool,
    button_config: WButtonConfig,
    show_keybinds: bool,
}

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
    CssReadError(PathBuf, gtk4::glib::Error),
}

fn file_search_given(given_file: impl AsRef<Path>) -> Result<PathBuf, WError> {
    let file = given_file.as_ref();
    if !file.is_file() {
        return Err(WError::SpecifiedPathNotAFile(file.to_owned()));
    }

    Ok(file.to_owned())
}

fn file_search_path(file_name: impl AsRef<Path>) -> Result<PathBuf, WError> {
    let file_name = file_name.as_ref();
    let user_config_dir = dirs::config_dir()
        .or_else(|| dirs::home_dir().map(|p| p.join(".config")))
        .unwrap_or_else(|| Path::new("~/.config").to_owned());

    for path in &[
        &user_config_dir.join("wleave"),
        &user_config_dir.join("wlogout"),
        Path::new("/etc/wleave"),
        Path::new("/etc/wlogout"),
        Path::new("/usr/local/etc/wleave"),
        Path::new("/usr/local/etc/wlogout"),
    ] {
        let full_path = path.join(file_name);
        if full_path.is_file() {
            info!("File found in: {}", full_path.display());
            return Ok(full_path);
        } else {
            info!("No file found in: {}", full_path.display());
        }
    }

    Err(WError::FileNotInSearchPath(file_name.to_owned()))
}

fn parse_config(input: impl Read, source_path: Cow<Path>) -> Result<WButtonConfig, WError> {
    let path = source_path.into_owned();
    let path_name = path.display().to_string();
    info!("Reading options from: {}", path_name);
    let config = std::io::read_to_string(input).map_err(|e| WError::IoError(path, e))?;

    let new = serde_json::de::from_str::<WButtonConfig>(&config).map_err(|e| {
        WError::FileParseFailed(
            NamedSource::new(path_name.clone(), config.to_owned()),
            SourceOffset::from_location(&config, e.line(), e.column()),
            e,
        )
    });

    let legacy = serde_json::Deserializer::from_str(&config)
        .into_iter::<WButton>()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            WError::FileParseFailed(
                NamedSource::new(path_name, config.to_owned()),
                SourceOffset::from_location(&config, e.line(), e.column()),
                e,
            )
        })
        .map(|buttons| WButtonConfig { buttons });

    match (new, legacy) {
        (Ok(conf), _) => {
            info!("Using the JSON layout format.");
            Ok(conf)
        }
        (Err(e), Ok(legacy)) => {
            info!("Using the backwards-compatible layout format.");
            if !enabled!(Level::DEBUG) {
                warn!( "If this is not intended, run the application with RUST_LOG=debug to show the JSON parse error.");
            }

            debug!("The JSON format could not be parsed: {:?}", Report::from(e));

            Ok(legacy)
        }
        (Err(e), Err(_)) => {
            error!("{:?}", e);

            Err(e)
        }
    }
}

fn load_config(file: Option<&impl AsRef<Path>>) -> Result<WButtonConfig, WError> {
    if let Some("-") = file.map(AsRef::as_ref).and_then(Path::to_str) {
        return parse_config(std::io::stdin(), Path::new("<stdin>").into());
    }

    let file_path = file.map(file_search_given).unwrap_or_else(|| {
        file_search_path("layout.json").or_else(|_| file_search_path("layout"))
    })?;

    let input =
        std::fs::File::open(&file_path).map_err(|e| WError::IoError(file_path.clone(), e))?;
    parse_config(input, file_path.into())
}

fn load_css(file: Option<impl AsRef<Path>>) -> Result<CssProvider, WError> {
    let path = file
        .map(file_search_given)
        .unwrap_or_else(|| file_search_path("style.css"))?;

    let provider = CssProvider::new();
    provider.connect_parsing_error(|_provider, _section, error| {
        warn!("CSS Parse error: {:?}", error);
    });
    provider.load_from_file(&gio::File::for_path(&path));

    Ok(provider)
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

    window.hide();
}

fn handle_key(
    config: &Arc<AppConfig>,
    window: &ApplicationWindow,
    key: &gtk4::gdk::Key,
) -> Propagation {
    if let &Key::Escape = key {
        window.close();
        return Propagation::Proceed;
    }

    let key = key
        .to_unicode()
        .map(|c| c.to_string())
        .or_else(|| key.name().map(|s| s.to_string()));

    if let Some(ref key_name) = key {
        let button = config
            .button_config
            .buttons
            .iter()
            .find(|b| b.keybind == *key_name);

        if let Some(WButton { action, .. }) = button {
            let state_action = action.clone();
            on_option(&state_action, config.delay_ms, window.clone());
        }
    }

    Propagation::Proceed
}

fn app_main(config: &Arc<AppConfig>, app: &Application) {
    let grid = gtk4::Grid::new();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("wleave")
        .child(&grid)
        .build();

    match config.protocol {
        Protocol::LayerShell => {
            window.init_layer_shell();
            window.set_layer(gtk4_layer_shell::Layer::Overlay);
            window.set_namespace("wleave");
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
            "center" => gtk4::Justification::Center,
            "fill" => gtk4::Justification::Fill,
            "left" => gtk4::Justification::Left,
            "right" => gtk4::Justification::Right,
            _ => gtk4::Justification::Center,
        };

        let button = gtk4::Button::builder()
            .label(&label)
            .name(&bttn.label)
            .hexpand(true)
            .vexpand(true)
            .cursor(&Cursor::from_name("pointer", None).expect("pointer cursor not found"))
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

        button.connect_clicked(clone!(
            #[weak]
            window,
            #[to_owned(rename_to = action)]
            &bttn.action,
            #[to_owned(rename_to = delay_ms)]
            &config.delay_ms,
            #[upgrade_or_panic]
            move |_| on_option(&action, delay_ms, window)
        ));

        let x = i as u32 % config.buttons_per_row;
        let y = i as u32 / config.buttons_per_row;

        grid.attach(&button, x as i32, y as i32, 1, 1);
    }

    window.present();
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

    let button_config = load_config(args.layout.as_ref())?;

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
        delay_ms: args.delay_command_ms,
    });

    let app = Application::builder()
        .application_id("sh.natty.Wleave")
        .build();

    app.connect_startup(move |_| match load_css(args.css.as_ref()) {
        Ok(css) => gtk4::style_context_add_provider_for_display(
            &Display::default().expect("Could not connect to a display"),
            &css,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        ),
        Err(e) => error!("Failed to load CSS: {e}"),
    });

    app.connect_activate(move |app| app_main(&config, app));

    app.run_with_args(&[] as &[&str]);

    Ok(())
}
