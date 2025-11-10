use crate::WError;
use crate::button::WButton;
use gdk4::gio;
use gtk4::CssProvider;
use miette::{NamedSource, Report, SourceOffset};
use serde::Deserialize;
use std::borrow::Cow;
use std::io::Read;
use std::path::{Path, PathBuf};
use tracing::{Level, debug, enabled, error, info, warn};
use wleave::cli_opt::{Args, AspectRatio, ButtonLayout, Protocol};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AppConfig {
    pub margin_left: Option<i32>,
    pub margin_right: Option<i32>,
    pub margin_top: Option<i32>,
    pub margin_bottom: Option<i32>,
    #[serde(default = "default_margin")]
    pub margin: i32,
    #[serde(default = "default_spacing")]
    pub column_spacing: u32,
    #[serde(default = "default_spacing")]
    pub row_spacing: u32,
    pub button_aspect_ratio: Option<AspectRatio>,
    #[serde(default = "default_delay")]
    pub delay_command_ms: u32,
    #[serde(default)]
    pub protocol: Protocol,
    #[serde(default)]
    pub buttons_per_row: ButtonLayout,
    #[serde(default)]
    pub close_on_lost_focus: bool,
    pub buttons: Vec<WButton>,
    #[serde(default)]
    pub show_keybinds: bool,
    #[serde(default)]
    pub no_version_info: bool,
    pub css: Option<PathBuf>,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            margin_left: None,
            margin_right: None,
            margin_top: None,
            margin_bottom: None,
            margin: default_margin(),
            column_spacing: default_spacing(),
            row_spacing: default_spacing(),
            button_aspect_ratio: None,
            delay_command_ms: default_delay(),
            protocol: Default::default(),
            buttons_per_row: Default::default(),
            close_on_lost_focus: false,
            buttons: vec![],
            show_keybinds: false,
            no_version_info: false,
            css: None,
        }
    }
}

fn default_margin() -> i32 {
    200
}

fn default_spacing() -> u32 {
    8
}

fn default_delay() -> u32 {
    100
}

fn file_search_given(given_file: impl AsRef<Path>) -> Result<PathBuf, WError> {
    let file = given_file.as_ref();
    if !file.is_file() {
        return Err(WError::SpecifiedPathNotAFile(file.to_owned()));
    }

    Ok(file.to_owned())
}

pub fn file_search_path(file_name: impl AsRef<Path>) -> Result<PathBuf, WError> {
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
            debug!("File found in: {}", full_path.display());
            return Ok(full_path);
        } else {
            debug!("No file found in: {}", full_path.display());
        }
    }

    Err(WError::FileNotInSearchPath(file_name.to_owned()))
}

fn parse_config(input: impl Read, source_path: Cow<Path>) -> Result<AppConfig, WError> {
    let path = source_path.into_owned();
    let path_name = path.display().to_string();
    info!("Reading options from: {}", path_name);
    let config = std::io::read_to_string(input).map_err(|e| WError::IoError(path, e))?;

    let new = serde_json::de::from_str::<AppConfig>(&config).map_err(|e| {
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
        .map(|buttons| AppConfig {
            buttons,
            ..Default::default()
        });

    match (new, legacy) {
        (Ok(conf), _) => {
            info!("Using the JSON layout format.");
            Ok(conf)
        }
        (Err(e), Ok(legacy)) => {
            debug!("The JSON format could not be parsed: {:?}", Report::from(e));
            info!("Using the backwards-compatible layout format.");
            if !enabled!(Level::DEBUG) {
                warn!(
                    "If this is not intended, run the application with RUST_LOG=debug to show the JSON parse error."
                );
            }

            Ok(legacy)
        }
        (Err(e), Err(_)) => {
            error!("{:?}", e);

            Err(e)
        }
    }
}

pub fn load_config(file: Option<&impl AsRef<Path>>) -> Result<AppConfig, WError> {
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

pub fn load_css(file: Option<impl AsRef<Path>>) -> Result<CssProvider, WError> {
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

pub fn merge_with_args(config: &mut AppConfig, args: &Args) {
    if let Some(margin_top) = args.margin_top {
        info!("\"margin-top\" specified from args: {}", margin_top);
        config.margin_top = Some(margin_top);
    } else {
        info!(
            "\"margin-top\" specified from config: {:?}",
            config.margin_top
        );
    }

    if let Some(margin_bottom) = args.margin_bottom {
        info!("\"margin-bottom\" specified from args: {}", margin_bottom);
        config.margin_bottom = Some(margin_bottom);
    } else {
        info!(
            "\"margin-bottom\" specified from config: {:?}",
            config.margin_bottom
        );
    }

    if let Some(margin_left) = args.margin_left {
        info!("\"margin-left\" specified from args: {}", margin_left);
        config.margin_left = Some(margin_left);
    } else {
        info!(
            "\"margin-left\" specified from config: {:?}",
            config.margin_left
        );
    }

    if let Some(margin_right) = args.margin_right {
        info!("\"margin-right\" specified from args: {}", margin_right);
        config.margin_right = Some(margin_right);
    } else {
        info!(
            "\"margin-right\" specified from config: {:?}",
            config.margin_right
        );
    }

    if let Some(margin) = args.margin {
        info!("\"margin\" specified from args: {}", margin);
        config.margin = margin;
    } else {
        info!("\"margin\" specified from config: {}", config.margin);
    }

    if let Some(protocol) = args.protocol {
        info!("\"protocol\" specified from args: {:?}", protocol);
        config.protocol = protocol;
    } else {
        info!("\"protocol\" specified from config: {:?}", config.protocol);
    }

    if let Some(column_spacing) = args.column_spacing {
        info!("\"column-spacing\" specified from args: {}", column_spacing);
        config.column_spacing = column_spacing;
    } else {
        info!(
            "\"column-spacing\" specified from config: {}",
            config.column_spacing
        );
    }

    if let Some(row_spacing) = args.row_spacing {
        info!("\"row-spacing\" specified from args: {}", row_spacing);
        config.row_spacing = row_spacing;
    } else {
        info!(
            "\"row-spacing\" specified from config: {}",
            config.row_spacing
        );
    }

    if let Some(aspect_ratio) = args.button_aspect_ratio {
        info!(
            "\"button-aspect-ratio\" specified from args: {}",
            aspect_ratio
        );
        config.button_aspect_ratio = Some(aspect_ratio);
    } else {
        info!(
            "\"button-aspect-ratio\" specified from config: {:?}",
            config.button_aspect_ratio
        );
    }

    if let Some(show_keybinds) = args.show_keybinds {
        info!("\"show-keybinds\" specified from args: {}", show_keybinds);
        config.show_keybinds = show_keybinds;
    } else {
        info!(
            "\"show-keybinds\" specified from config: {}",
            config.show_keybinds
        );
    }

    if let Some(close_on_lost_focus) = args.close_on_lost_focus {
        info!(
            "\"close-on-lost-focus\" specified from args: {}",
            close_on_lost_focus
        );
        config.close_on_lost_focus = close_on_lost_focus;
    } else {
        info!(
            "\"close-on-lost-focus\" specified from config: {}",
            config.close_on_lost_focus
        );
    }

    if let Some(buttons_per_row) = args.buttons_per_row {
        info!(
            "\"buttons-per-row\" specified from args: {:?}",
            buttons_per_row
        );
        config.buttons_per_row = buttons_per_row;
    } else {
        info!(
            "\"buttons-per-row\" specified from config: {:?}",
            config.buttons_per_row
        );
    }

    if let Some(no_version_info) = args.no_version_info {
        info!(
            "\"no-version-info\" specified from args: {}",
            no_version_info
        );
        config.no_version_info = no_version_info;
    } else {
        info!(
            "\"no-version-info\" specified from config: {}",
            config.no_version_info
        );
    }

    if let Some(delay_command_ms) = args.delay_command_ms {
        info!(
            "\"delay-command-ms\" specified from args: {}",
            delay_command_ms
        );
        config.delay_command_ms = delay_command_ms;
    } else {
        info!(
            "\"delay-command-ms\" specified from config: {}",
            config.delay_command_ms
        );
    }

    if let Some(no_version_info) = args.no_version_info {
        info!(
            "\"no-version-info\" specified from args: {}",
            no_version_info
        );
        config.no_version_info = no_version_info;
    } else {
        info!(
            "\"no-version-info\" specified from config: {}",
            config.no_version_info
        );
    }

    if let Some(css) = args.css.clone() {
        info!("\"css\" file specified from args: {:?}", css.display());
        config.css = Some(css);
    } else {
        info!(
            "\"css\" file specified from config: {:?}",
            config.css.as_deref().map(Path::display)
        );
    }
}
