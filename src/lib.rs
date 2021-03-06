#[macro_use]
extern crate vst;
extern crate vst_gui;
extern crate lerp;
extern crate math;

use std::sync::{Arc, Mutex};
use std::f32::consts::PI;
use math::round;

use vst::buffer::AudioBuffer;
use vst::editor::Editor;
use vst::plugin::{Category, Plugin, Info};

use lerp::Lerp;

const HTML: &'static str = include_str!("./gui.html");

struct Parameters {
    pub threshold: f32
}

fn create_javascript_callback(
    oscillator: Arc<Mutex<Parameters>>) -> vst_gui::JavascriptCallback
{
    Box::new(move |message: String| {
        let mut tokens = message.split_whitespace();

        let command = tokens.next().unwrap_or("");
        let argument = tokens.next().unwrap_or("").parse::<f32>();

        let mut locked_oscillator = oscillator.lock().unwrap();

        match command {
            "getThreshold" => {
                return locked_oscillator.threshold.to_string();
            },
            "setThreshold" => {
                if argument.is_ok() {
                    locked_oscillator.threshold = argument.unwrap();
                }
            },
            _ => {}
        }

        String::new()
    })
}

struct BleachInjector {
    sample_rate: f32,
    time: f32,
    // We access this object both from a UI thread and from an audio processing
    // thread.
    params: Arc<Mutex<Parameters>>,
}

impl Default for BleachInjector {
    fn default() -> BleachInjector {
        let params = Arc::new(Mutex::new(
            Parameters {
                threshold: 1.0
            }
        ));

        BleachInjector {
            sample_rate: 44100.0,
            time: 0.0,
            params: params.clone(),
        }
    }
}

impl Plugin for BleachInjector {
    fn get_info(&self) -> Info {
        Info {
            name: "BleachInjector".to_string(),
            vendor: "Rein van der Woerd".to_string(),
            unique_id: 25032017,

            inputs: 2,
            outputs: 2,
            parameters: 0,
            category: Category::Effect,

            ..Info::default()
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate as f32;
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let params = self.params.lock().unwrap();

        let per_step = 1.0 / self.sample_rate;

        for (input, output) in buffer.zip() {
            // For each input sample and output sample in buffer
            for (in_frame, out_frame) in input.into_iter().zip(output.into_iter()) {
                self.time += per_step;
                let distorted;
                let tremolo = (self.time*300.0).sin() * 0.5 + 0.5;
                 
                distorted = in_frame.min(params.threshold).max(-params.threshold) / (params.threshold * 1.25);

                *out_frame = round::ceil(distorted as f64, (params.threshold * 3.0) as i8) as f32;
                // * (1.0 - tremolo * params.threshold);
            }
        }
    }

    fn get_editor(&mut self) -> Option<Box<dyn Editor>> {
        let gui = vst_gui::new_plugin_gui(
            String::from(HTML),
            create_javascript_callback(self.params.clone()),
            Some((400, 600)));
        Some(Box::new(gui))
    }
}

plugin_main!(BleachInjector);
