use nih_plug::{params, prelude::*};
use std::{sync::Arc, usize};

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

struct Myplug {
    params: Arc<MyplugParams>,
    prevsample: Vec<f32>,
    iterdelay: usize,
    iterrepeats: usize,
    prev: usize,
}

#[derive(Params)]
struct MyplugParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
    #[id = "gain"]
    pub gain: FloatParam,
    #[id = "delay"]
    pub delay: IntParam,
    #[id = "mode"]
    pub mode: IntParam,
    #[id = "time"]
    pub time: IntParam,
}

impl Default for Myplug {
    fn default() -> Self {
        Self {
            params: Arc::new(MyplugParams::default()),
            prevsample: vec![0.0; 400000],
            iterdelay: 0,
            iterrepeats: 399999,
            prev: 399999,
        }
    }
}

impl Default for MyplugParams {
    fn default() -> Self {
        Self {
            // This gain is stored as linear gain. NIH-plug comes with useful conversion functions
            // to treat these kinds of parameters as if we were dealing with decibels. Storing this
            // as decibels is easier to work with, but requires a conversion for every sample.
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    // This makes the range appear as if it was linear when displaying the values as
                    // decibels
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            // Because the gain parameter is stored as linear gain instead of storing the value as
            // decibels, we need logarithmic smoothing
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            // There are many predefined formatters we can use here. If the gain was stored as
            // decibels instead of as a linear gain value, we could have also used the
            // `.with_step_size(0.1)` function to get internal rounding.
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            delay: IntParam::new("Delay", 0, IntRange::Linear { min: 1, max: 1000 })
                .with_smoother(SmoothingStyle::None),
            mode: IntParam::new("Mode", 1, IntRange::Linear { min: 1, max: 7 })
                .with_smoother(SmoothingStyle::None),
            time: IntParam::new("Time", 1, IntRange::Linear { min: 1, max: 1000 })
                .with_smoother(SmoothingStyle::None),
        }
    }
}

impl Plugin for Myplug {
    const NAME: &'static str = "Myplug2.1";
    const VENDOR: &'static str = "SnovaxZ";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "snovaxz@proton.me";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            // Smoothing is optionally built into the parameters themselves
            let gain = self.params.gain.smoothed.next();
            let mut prevsample;
            let mut prevsample2;
            if self.prev != self.params.time.smoothed.next() as usize {
                self.iterrepeats = (self.iterrepeats as f32
                    * self.params.time.smoothed.next() as f32
                    / 1000.0) as usize;
                self.prev = self.params.time.smoothed.next() as usize;
            }
            for sample in channel_samples {
                prevsample = self.prevsample[self.iterrepeats];
                prevsample2 = self.prevsample[(self.iterrepeats as f32
                    * self.params.delay.smoothed.next() as f32
                    / 1000.0) as usize];
                *sample *= gain;
                self.prevsample[self.iterdelay] = *sample;
                self.iterdelay += 1;
                self.iterrepeats += 1;
                match self.params.mode.smoothed.next() {
                    1 => {
                        *sample += prevsample;
                    }
                    2 => {
                        *sample += prevsample;
                        if self.iterdelay > 199999 {
                            if self.iterdelay % 5 == 0 {
                                self.iterrepeats -= 1;
                            } else if self.iterdelay % 7 == 0 {
                                self.iterrepeats += 2;
                            };
                        } else {
                            self.iterrepeats += 1;
                        };
                    }
                    3 => {
                        *sample = prevsample;
                    }
                    4 => {
                        *sample *= prevsample;
                    }
                    5 => {
                        *sample += prevsample + prevsample2;
                    }
                    6 => {
                        *sample += prevsample;
                        if self.iterdelay % 3 == 0 {
                            if self.iterdelay % 2 == 0 {
                                self.iterrepeats -= 3;
                            } else {
                                self.iterrepeats += 3;
                            };
                        };
                    }
                    7 => {
                        *sample += prevsample + prevsample2;
                        if self.iterdelay % 3 == 0 {
                            if self.iterdelay % 2 == 0 {
                                self.iterrepeats -= 3;
                            } else {
                                self.iterrepeats += 3;
                            };
                        };
                        if self.iterdelay > 199999 {
                            if self.iterdelay % 5 == 0 {
                                self.iterrepeats -= 1;
                            } else if self.iterdelay % 7 == 0 {
                                self.iterrepeats += 2;
                            };
                        } else {
                            self.iterrepeats += 1;
                        }
                    }
                    _ => {}
                };
                if self.iterdelay >= 399999 {
                    self.iterdelay = 0;
                };
                if self.iterrepeats >= 399999 {
                    self.iterrepeats = 0;
                };
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Myplug {
    const CLAP_ID: &'static str = "com.your-domain.MYPLUG";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Testplugin for fun");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for Myplug {
    const VST3_CLASS_ID: [u8; 16] = *b"Myplug__________";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(Myplug);
nih_export_vst3!(Myplug);
