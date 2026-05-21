use std::sync::{LazyLock, Mutex};

use gst::glib;
use gst::prelude::*;
use gst::subclass::prelude::*;
use gst_base::subclass::prelude::*;

static CAT: LazyLock<gst::DebugCategory> = LazyLock::new(|| {
    gst::DebugCategory::new(
        "delay",
        gst::DebugColorFlags::empty(),
        Some("Fixed delay element (PTS/DTS shift)"),
    )
});

#[derive(Default)]
struct Settings {
    delay: gst::ClockTime,
}

#[derive(Default)]
pub struct Delay {
    settings: Mutex<Settings>,
}

#[glib::object_subclass]
impl ObjectSubclass for Delay {
    const NAME: &'static str = "GstDelay";
    type Type = super::Delay;
    type ParentType = gst_base::BaseTransform;
}

impl ObjectImpl for Delay {
    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: LazyLock<Vec<glib::ParamSpec>> = LazyLock::new(|| {
            vec![glib::ParamSpecUInt64::builder("delay")
                .nick("Delay")
                .blurb("Amount to delay buffers by, in nanoseconds")
                .default_value(0)
                .mutable_playing()
                .build()]
        });
        PROPERTIES.as_ref()
    }

    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            "delay" => {
                let ns: u64 = value.get().expect("delay must be u64");
                let mut settings = self.settings.lock().unwrap();
                settings.delay = gst::ClockTime::from_nseconds(ns);
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "delay" => {
                let settings = self.settings.lock().unwrap();
                settings.delay.nseconds().to_value()
            }
            _ => unimplemented!(),
        }
    }
}

impl GstObjectImpl for Delay {}

impl ElementImpl for Delay {
    fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
        static METADATA: LazyLock<gst::subclass::ElementMetadata> = LazyLock::new(|| {
            gst::subclass::ElementMetadata::new(
                "Delay",
                "Generic",
                "Delays buffers by a fixed amount by shifting PTS/DTS",
                "Erik Herz <erik@vivoh.com>",
            )
        });
        Some(&*METADATA)
    }

    fn pad_templates() -> &'static [gst::PadTemplate] {
        static PAD_TEMPLATES: LazyLock<Vec<gst::PadTemplate>> = LazyLock::new(|| {
            let caps = gst::Caps::new_any();
            vec![
                gst::PadTemplate::new(
                    "sink",
                    gst::PadDirection::Sink,
                    gst::PadPresence::Always,
                    &caps,
                )
                .unwrap(),
                gst::PadTemplate::new(
                    "src",
                    gst::PadDirection::Src,
                    gst::PadPresence::Always,
                    &caps,
                )
                .unwrap(),
            ]
        });
        PAD_TEMPLATES.as_ref()
    }
}

impl BaseTransformImpl for Delay {
    const MODE: gst_base::subclass::BaseTransformMode =
        gst_base::subclass::BaseTransformMode::AlwaysInPlace;
    const PASSTHROUGH_ON_SAME_CAPS: bool = false;
    const TRANSFORM_IP_ON_PASSTHROUGH: bool = false;

    fn transform_ip(
        &self,
        buf: &mut gst::BufferRef,
    ) -> Result<gst::FlowSuccess, gst::FlowError> {
        let delay = self.settings.lock().unwrap().delay;
        if delay.is_zero() {
            return Ok(gst::FlowSuccess::Ok);
        }

        if let Some(pts) = buf.pts() {
            buf.set_pts(pts.checked_add(delay));
        }
        if let Some(dts) = buf.dts() {
            buf.set_dts(dts.checked_add(delay));
        }

        gst::trace!(
            CAT,
            imp = self,
            "Shifted buffer by {}: pts={:?} dts={:?}",
            delay,
            buf.pts(),
            buf.dts()
        );

        Ok(gst::FlowSuccess::Ok)
    }

    fn src_event(&self, event: gst::Event) -> bool {
        // QoS events travel upstream. Downstream sees buffers with PTS shifted
        // by +delay, so its QoS running-time references are also +delay
        // relative to what upstream produced. Translate back by subtracting
        // delay so upstream sees consistent timing feedback.
        let event = match event.view() {
            gst::EventView::Qos(qos) => {
                let delay = self.settings.lock().unwrap().delay;
                if delay.is_zero() {
                    event
                } else {
                    let (qtype, proportion, diff, timestamp) = qos.get();
                    let new_ts = timestamp.and_then(|t| t.checked_sub(delay));
                    gst::event::Qos::builder(qtype, proportion, diff)
                        .timestamp(new_ts)
                        .build()
                }
            }
            _ => event,
        };
        self.parent_src_event(event)
    }
}
