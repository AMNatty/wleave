use clap::{ArgAction, Parser, ValueEnum};
use serde::{Deserialize, Deserializer};
use std::{
    error::Error,
    fmt::{Debug, Display},
    num::NonZeroU32,
    path::PathBuf,
    str::FromStr,
};

#[derive(Debug, Copy, Clone, Default, ValueEnum, Deserialize)]
pub enum Protocol {
    #[default]
    LayerShell,
    Xdg,
}

#[derive(Parser, Debug)]
#[command(author, version, disable_version_flag = true, about, long_about = None)]
pub struct Args {
    #[arg(short = 'v', long, action = ArgAction::Version)]
    pub version: Option<bool>,

    /// Specify a layout file, specifying - will read the layout config from stdin
    #[arg(short = 'l', long)]
    pub layout: Option<PathBuf>,

    /// Specify a custom CSS file
    #[arg(short = 'C', long)]
    pub css: Option<PathBuf>,

    /// Set the number of buttons per row, or use a fraction to specify the number of rows to be
    /// used (e.g. "1/1" for all buttons in a single row, "1/5" to distribute the buttons over 5 rows)
    #[arg(short = 'b', long, value_parser = clap::value_parser!(ButtonLayout))]
    pub buttons_per_row: Option<ButtonLayout>,

    /// Set space between buttons columns
    #[arg(short = 'c', long)]
    pub column_spacing: Option<u32>,

    /// Set space between buttons rows
    #[arg(short = 'r', long)]
    pub row_spacing: Option<u32>,

    /// Set the margin around buttons
    #[arg(short = 'm', long)]
    pub margin: Option<i32>,

    /// Set margin for the left of buttons
    #[arg(short = 'L', long)]
    pub margin_left: Option<i32>,

    /// Set margin for the right of buttons
    #[arg(short = 'R', long)]
    pub margin_right: Option<i32>,

    /// Set margin for the top of buttons
    #[arg(short = 'T', long)]
    pub margin_top: Option<i32>,

    /// Set the margin for the bottom of buttons
    #[arg(short = 'B', long)]
    pub margin_bottom: Option<i32>,

    /// The delay (in milliseconds) between the window closing and executing the selected option
    #[arg(short = 'd', long)]
    pub delay_command_ms: Option<u32>,

    /// Close the menu on lost focus
    #[arg(short = 'f', long, default_missing_value = "true")]
    pub close_on_lost_focus: Option<bool>,

    /// Show the associated key binds
    #[arg(short = 'k', long, default_missing_value = "true")]
    pub show_keybinds: Option<bool>,

    /// Use layer-shell or xdg protocol
    #[arg(short = 'p', long, value_enum)]
    pub protocol: Option<Protocol>,

    /// Hide version information
    #[arg(short = 'x', long, default_missing_value = "true")]
    pub no_version_info: Option<bool>,
}

#[derive(Clone, Copy, Debug)]
pub enum ButtonLayout {
    PerRow(u32),
    RowRatio(u32, u32),
}

impl Default for ButtonLayout {
    fn default() -> Self {
        ButtonLayout::PerRow(3)
    }
}

impl<'de> Deserialize<'de> for ButtonLayout {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl FromStr for ButtonLayout {
    type Err = Box<dyn Error + Send + Sync + 'static>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(per_row) = s.parse::<NonZeroU32>() {
            return Ok(ButtonLayout::PerRow(per_row.into()));
        }

        if let Some((n, d)) = s.split_once("/") {
            if let (Ok(n), Ok(d)) = (n.parse::<NonZeroU32>(), d.parse::<NonZeroU32>()) {
                return Ok(ButtonLayout::RowRatio(n.into(), d.into()));
            }
        }

        Err("Value neither a number (1, 2, 3) nor a ratio (1/1, 2/3, ...)".into())
    }
}

impl Display for ButtonLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PerRow(r) => write!(f, "{r}"),
            Self::RowRatio(n, d) => write!(f, "{n}/{d}"),
        }
    }
}
