use gst::glib;
use gst::prelude::*;

mod imp;

glib::wrapper! {
    pub struct Delay(ObjectSubclass<imp::Delay>)
        @extends gst_base::BaseTransform, gst::Element, gst::Object;
}

pub fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    gst::Element::register(Some(plugin), "delay", gst::Rank::NONE, Delay::static_type())
}
