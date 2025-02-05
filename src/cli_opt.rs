use clap::{ArgAction, Parser, ValueEnum};
use std::{
    error::Error,
    fmt::{Debug, Display},
    num::NonZeroU32,
    path::PathBuf,
    str::FromStr,
};

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum Protocol {
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
    #[arg(short = 'b', long = "buttons-per-row", value_parser = clap::value_parser!(ButtonLayout), default_value_t = ButtonLayout::PerRow(3))]
    pub buttons_per_row: ButtonLayout,

    /// Set space between buttons columns
    #[arg(short = 'c', long = "column-spacing", default_value_t = 5)]
    pub column_spacing: u32,

    /// Set space between buttons rows
    #[arg(short = 'r', long = "row-spacing", default_value_t = 5)]
    pub row_spacing: u32,

    /// Set the margin around buttons
    #[arg(short = 'm', long, default_value_t = 230)]
    pub margin: i32,

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
    #[arg(short = 'd', long, default_value_t = 100)]
    pub delay_command_ms: u32,

    /// Close the menu on lost focus
    #[arg(short = 'f', long)]
    pub close_on_lost_focus: bool,

    /// Show the associated key binds
    #[arg(short = 'k', long)]
    pub show_keybinds: bool,

    /// Use layer-shell or xdg protocol
    #[arg(short = 'p', long, value_enum, default_value_t = Protocol::LayerShell)]
    pub protocol: Protocol,
}

#[derive(Clone, Copy, Debug)]
pub enum ButtonLayout {
    PerRow(u32),
    RowRatio(u32, u32),
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
