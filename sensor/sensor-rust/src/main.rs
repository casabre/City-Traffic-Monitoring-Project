use std::collections::HashMap;
use std::env;

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
    let id = get_id();
    let url = get_url();
    let topics = get_topics();
    let sample_rate = get_sample_rate();
    let mqtt = MqttPublisher::new(url, id, topics);
    let mut audio_sensor = Audio::new(|x| mqtt.append(x), sample_rate);
    audio_sensor.start();
    mqtt.start();
    // should not go here because MqttPublisher loop is blocking
}

fn get_id() -> String {
    let id = env::args()
        .nth(1)
        .unwrap_or_else(|| "sensor-data-abc".to_string());
    id
}

fn get_url() -> String {
    let url = env::args()
        .nth(1)
        .unwrap_or_else(|| "tcp://sctmp.ai:1883".to_string());
    url
}

fn get_topics() -> HashMap<String, String> {
    let mut topics = HashMap::new();
    topics.insert("out".to_string(), "sensor-data".to_string());
    topics
}

fn get_sample_rate() -> i32 {
    let default = 2_i32.pow(14);
    let sr = env::args()
        .nth(1)
        .unwrap_or_else(|| format!("{default}").to_string());
    sr.parse::<i32>().unwrap()
}
