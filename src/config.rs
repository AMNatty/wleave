use crate::button::WButton;
use crate::error::WError;
use convert_case::ccase;
use gdk4::gio;
use gtk4::CssProvider;
use miette::{NamedSource, Report, SourceOffset};
use serde::Deserialize;
use std::borrow::Cow;
use std::io::Read;
use std::path::{Path, PathBuf};
use tracing::{Level, debug, enabled, error, info, warn};
use wleave::cli_opt::{Args, ButtonLayout, MenuLayoutStrategy, Protocol};
use wleave::units::{AspectRatio, LengthValue, Margin};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AppConfig {
    #[serde(default)]
    pub service: bool,
    #[serde(default)]
    pub button_layout: MenuLayoutStrategy,
    pub default_button: Option<String>,
    pub margin_left: Option<Margin>,
    pub margin_right: Option<Margin>,
    pub margin_top: Option<Margin>,
    pub margin_bottom: Option<Margin>,
    #[serde(default = "default_margin")]
    pub margin: Margin,
    #[serde(default = "default_spacing")]
    pub column_spacing: LengthValue,
    #[serde(default = "default_spacing")]
    pub row_spacing: LengthValue,
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
            service: false,
            button_layout: MenuLayoutStrategy::Grid,
            default_button: None,
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

fn default_margin() -> Margin {
    Margin(LengthValue::Percentage(0.2))
}

fn default_spacing() -> LengthValue {
    LengthValue::Px(8.0)
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

macro_rules! merge_option {
    ($conf:ident, $args:ident, $name:ident => $val:expr) => {
        if let Some($name) = $args.$name {
            info!(
                "\"{}\" specified from args: {:?}",
                ccase!(snake -> kebab, stringify!($name)),
                $name
            );
            $conf.$name = $val;
        } else {
            info!(
                "\"{}\" specified from config: {:?}",
                ccase!(snake -> kebab, stringify!($name)),
                $conf.$name
            );
        }
    };
    ($conf:ident, $args:ident, $name:ident) => {
        merge_option!($conf, $args, $name => $name)
    };
}

pub fn merge_with_args(config: &mut AppConfig, args: Args) {
    merge_option!(config, args, service);
    merge_option!(config, args, button_layout);
    merge_option!(config, args, default_button => Some(default_button));
    merge_option!(config, args, margin_top => Some(margin_top));
    merge_option!(config, args, margin_bottom => Some(margin_bottom));
    merge_option!(config, args, margin_left => Some(margin_left));
    merge_option!(config, args, margin_right => Some(margin_right));
    merge_option!(config, args, margin);
    merge_option!(config, args, protocol);
    merge_option!(config, args, column_spacing);
    merge_option!(config, args, row_spacing);
    merge_option!(config, args, button_aspect_ratio => Some(button_aspect_ratio));
    merge_option!(config, args, show_keybinds);
    merge_option!(config, args, close_on_lost_focus);
    merge_option!(config, args, buttons_per_row);
    merge_option!(config, args, no_version_info);
    merge_option!(config, args, delay_command_ms);

    if let Some(css) = args.css.clone() {
        info!(
            "\"css\" file specified from args: {:?}",
            Path::display(&css)
        );
        config.css = Some(css);
    } else {
        info!(
            "\"css\" file specified from config: {:?}",
            config.css.as_deref().map(Path::display)
        );
    }
}
