use std::collections::HashMap;

mod audio;
mod publishing;
mod runner;
use crate::audio::Capturing;
use crate::publishing::Publishing;
use crate::runner::MutableRunner;
use crate::runner::Runner;
use audio::Audio;
use log::info;
use publishing::MqttPublisher;

fn main() {
    info!("Starting capturing and forwarding");
    let mqtt = MqttPublisher::new("".to_string(), "".to_string(), HashMap::new());
    let sample_rate: i32 = 2_i32.pow(14);
    let mut audio_sensor = Audio::new(|x| mqtt.append(x), sample_rate);
    audio_sensor.start();
    mqtt.start();
}
