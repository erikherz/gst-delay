use gst::glib;

mod delay;

fn plugin_init(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    delay::register(plugin)
}

gst::plugin_define!(
    delay,
    env!("CARGO_PKG_DESCRIPTION"),
    plugin_init,
    env!("CARGO_PKG_VERSION"),
    "MPL-2.0",
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_REPOSITORY"),
    "2026-05-20"
);
