use crate::runner::MutableRunner;
use serde_derive::Serialize;
use std::collections::HashMap;

pub trait Capturing<'a> {
    fn new<'b: 'a, Processor>(sound_forwarder: Processor, sample_rate: i32) -> Self
    where
        Processor: 'b + Fn(Box<dyn erased_serde::Serialize>);
}

#[derive(Serialize)]
struct AudioSenMl {
    pub raw: HashMap<usize, Vec<i16>>,
    pub raw_type: String,
    pub channel_count: usize,
    pub sample_count: usize,
    pub sample_rate: i32,
}

pub struct Audio<'a> {
    ctx: soundio::Context<'a>,
    input_stream: soundio::InStream<'a>,
}

impl<'a> MutableRunner<'a> for Audio<'a> {
    fn start(&mut self) {
        match self.input_stream.start() {
            Err(e) => panic!("Error starting stream: {}", e),
            Ok(f) => f,
        };
    }
    fn stop(&mut self) {}
}

impl<'a> Capturing<'a> for Audio<'a> {
    fn new<'b: 'a, Processor>(sound_forwarder: Processor, sample_rate: i32) -> Audio<'a>
    where
        Processor: 'b + Fn(Box<dyn erased_serde::Serialize>),
    {
        // Data preparation closure
        let read_callback = move |stream: &mut soundio::InStreamReader| {
            let frame_count_max = stream.frame_count_max();
            if let Err(e) = stream.begin_read(frame_count_max) {
                println!("Error reading from stream: {}", e);
                return;
            }

            let frames: usize = stream.frame_count();
            let channels: usize = stream.channel_count();
            let mut map = HashMap::new();
            let getter = |cc, ff| {
                let data = stream.sample::<i16>(cc, ff);
                data
            };
            for f in 0..frames {
                for c in 0..channels {
                    map.entry(c)
                        .and_modify(|e: &mut Vec<i16>| e.push(getter(c, f)))
                        .or_insert({
                            let mut vec = Vec::new();
                            vec.push(getter(c, f));
                            vec
                        });
                }
            }
            let ml = AudioSenMl {
                raw: map,
                raw_type: "S16LE".to_string(),
                channel_count: channels,
                sample_count: frames,
                sample_rate: sample_rate,
            };
            let bml = Box::new(ml);
            sound_forwarder(bml);
        };

        // Initialize soundio
        let ctx = create_ctx().unwrap();
        let input_stream: soundio::InStream;
        {
            let dev = create_dev(&ctx, sample_rate).unwrap();
            input_stream = match dev.open_instream(
                sample_rate,
                soundio::Format::S16LE,
                soundio::ChannelLayout::get_builtin(soundio::ChannelLayoutId::Stereo),
                1.0,
                read_callback,
                None::<fn()>,
                None::<fn(soundio::Error)>,
            ) {
                Err(e) => panic!("Error creating stream: {}", e),
                Ok(f) => f,
            };
        }

        Audio {
            ctx: ctx,
            input_stream: input_stream,
        }
    }
}

fn create_ctx<'a>() -> Result<soundio::Context<'a>, soundio::Error> {
    let mut ctx = soundio::Context::new();
    ctx.set_app_name("Recorder");
    match ctx.connect() {
        Err(e) => panic!("Error connecting soundio context: {}", e),
        Ok(f) => f,
    };
    ctx.flush_events();
    Ok(ctx)
}

fn create_dev<'a>(
    ctx: &'a soundio::Context,
    sample_rate: i32,
) -> Result<soundio::Device<'a>, soundio::Error> {
    let dev = ctx.default_input_device().expect("No input device");
    if !dev.supports_layout(soundio::ChannelLayout::get_builtin(
        soundio::ChannelLayoutId::Stereo,
    )) {
        panic!("Device doesn't support stereo");
    }
    if !dev.supports_format(soundio::Format::S16LE) {
        panic!("Device doesn't support S16LE");
    }
    if !dev.supports_sample_rate(sample_rate) {
        let khz: f32 = sample_rate as f32 / 1000.0;
        panic!("Device doesn't support {khz} kHz");
    }
    Ok(dev)
}
