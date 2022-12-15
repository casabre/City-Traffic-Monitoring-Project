pub trait Capturing<'a> {
    fn new<'b: 'a, Processor>(&self, sound_processor: Processor, sample_rate: i32) -> Self
    where
        Processor: 'b + Fn(Vec<i16>);
    fn run(&mut self);
}

pub struct Audio<'a> {
    input_stream: soundio::InStream<'a>,
}

impl<'a> Capturing<'a> for Audio<'a> {
    fn new<'b: 'a, Processor>(&self, sound_processor: Processor, sample_rate: i32) -> Audio<'a>
    where
        Processor: 'b + Fn(Vec<i16>),
    {
        let mut ctx: soundio::Context<'a> = soundio::Context::new();
        ctx.set_app_name("Recorder");
        match ctx.connect() {
            Err(e) => panic!("Error connecting soundio context: {}", e),
            Ok(f) => f,
        };
        ctx.flush_events();

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
            panic!("Device doesn't {khz} kHz");
        }

        let read_callback = move |stream: &mut soundio::InStreamReader| {
            let frame_count_max = stream.frame_count_max();
            if let Err(e) = stream.begin_read(frame_count_max) {
                println!("Error reading from stream: {}", e);
                return;
            }

            let frames: usize = stream.frame_count();
            let channels: usize = stream.channel_count();
            let size: usize = frames * channels;
            let mut vector: Vec<i16> = vec![0; size];
            for f in 0..frames {
                for c in 0..channels {
                    let m = c * f;
                    vector[f + m] = stream.sample::<i16>(c, f);
                }
            }
            sound_processor(vector);
        };

        let input_stream = match dev.open_instream(
            44100,
            soundio::Format::S16LE,
            soundio::ChannelLayout::get_builtin(soundio::ChannelLayoutId::Stereo),
            2.0,
            read_callback,
            None::<fn()>,
            None::<fn(soundio::Error)>,
        ) {
            Err(e) => panic!("Error creating stream: {}", e),
            Ok(f) => f,
        };

        Audio {
            input_stream: input_stream,
        }
    }

    fn run(&mut self) {
        match self.input_stream.start() {
            Err(e) => panic!("Error starting stream: {}", e),
            Ok(f) => f,
        };
    }
}
