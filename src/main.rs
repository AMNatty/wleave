mod app;
mod button;
mod config;
mod error;
mod exec;
mod layout;
mod paintable;

use clap::Parser;
use glib::clone;
use std::sync::Arc;
use tracing::{Level, error};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::app::create_app;
use crate::config::{AppConfig, load_config, load_css, merge_with_args};
use gtk4::gdk::Display;
use gtk4::prelude::*;
use wleave::cli_opt::Args;

fn on_startup(config: &AppConfig) {
    let display = Display::default().expect("Could not connect to a display");

    match load_css(config.css.as_deref()) {
        Ok(css) => gtk4::style_context_add_provider_for_display(
            &display,
            &css,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        ),
        Err(e) => error!("Failed to load CSS: {e}"),
    };
}

fn entry_point(config: Arc<AppConfig>) -> miette::Result<()> {
    let mut flags = gtk4::gio::ApplicationFlags::empty();
    flags.set(gtk4::gio::ApplicationFlags::IS_SERVICE, config.service);

    let app = libadwaita::Application::builder()
        .application_id("sh.natty.Wleave")
        .flags(flags)
        .build();

    app.connect_startup(clone!(
        #[strong]
        config,
        move |_| on_startup(config.as_ref())
    ));

    let hold_guard = if config.service {
        Some(app.hold())
    } else {
        None
    };

    app.connect_activate(move |app| {
        let _ = &hold_guard;

        let app_window = create_app(&config, app);
        app_window.present();
    });

    app.run_with_args(&[] as &[&str]);

    Ok(())
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

    let mut config = load_config(args.layout.as_deref())?;
    merge_with_args(&mut config, args);

    let config = Arc::new(config);
    entry_point(config)?;

    Ok(())
}
