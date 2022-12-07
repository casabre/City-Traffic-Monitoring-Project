use log::{as_error, info, trace, warn};
use std::{env, process, time::Duration};
use bytebuffer::{ByteBuffer};
use std::collections::HashMap;
use async_trait::async_trait;
use futures::executor::block_on;

extern crate paho_mqtt as mqtt;

#[async_trait]
pub trait Publishing {
    pub fn new(url: String, id: String, topics: [String]) -> Self;
    async fn publish(&self, msg: HashMap);
    pub fn append(&self, data: [char])
    pub fn run(&self);
    async fn run_loop(&self) 
    fn fetch(&self) -> &[u8];
}

pub struct MqttPublisher {
    url: String,
    id: String,
    topics: [String],
    client: mqtt::Client,
    buffer: ByteBuffer,
    run: bool,
    buffer_array_size: i32
}

impl Publishing for MqttPublisher {
    fn new(url: String, id: String, topics: [String]) -> MqttPublisher {
        // Define the set of options for the create.
        // Use an ID for a persistent session.
        let create_opts = mqtt::CreateOptionsBuilder::new()
            .server_uri(url)
            .client_id(id)
            .finalize();

        // Create a client.
        let client = mqtt::Client::new(create_opts).unwrap_or_else(|err| {
            as_error!("Error creating the client: {:?}", err);
        });

        // Define the set of options for the connection.
        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(20))
            .clean_session(true)
            .finalize();

        // Connect and wait for it to complete or fail.
        if let Err(e) = client.connect(conn_opts) {
            as_error!("Unable to connect:\n\t{:?}", e);
        }

        info!("Successfully created client for host {:?}", url)

        MqttPublisher {
            url: url,
            id: id,
            topics: topics,
            client: client,
            buffer: ByteBuffer::new(),
            run: true,
            buffer_array_size: -1
        }
    }

    fn set_audio_samples_as_u8(&self, samples: u32) {
        self.buffer_array_size = samples;
    }

    fn run(&self) {
        block_on(self.run_loop())
    }

    fn run_loop(&self) {
        loop (true) {
            let stream = self.fetch();
        }
    }

    fn append(&self, data: &[u8]) {
        self.buffer.write_bytes(data);
    }

    fn fetch(&self) -> Vec<u8> {
        let bytes = self.buffer.read_bytes(self.buffer_array_size);
        return bytes;
    }

    fn prepare_dataset(&self, samples: Vec<u8>) -> HashMap {

    }

    fn publish(&self, msg: HashMap) {
        let tok = self.client.publish(msg);
        if let Err(e) = tok {
            as_error!("Error sending message: {:?}", e);
        }
    }
}
