use crossbeam::queue::SegQueue;
use erased_serde::Serialize;
use log::{error, info};
use serde_cbor::to_vec;
use std::collections::HashMap;
use std::{thread, time};

use crate::runner::Runner;
extern crate paho_mqtt as mqtt;

pub trait Publishing<'a> {
    fn new(url: String, id: String, topics: HashMap<String, String>) -> Self;
    fn append(&self, value: Box<dyn Serialize>);
    fn fetch(&self) -> Option<Box<dyn Serialize>>;
    fn publish(&self, msg: Vec<u8>);
}

pub struct MqttPublisher {
    topics: HashMap<String, String>,
    client: mqtt::Client,
    buffer: SegQueue<Box<dyn Serialize>>,
    qos: i32,
}

impl<'a> Runner<'a> for MqttPublisher {
    fn start(&self) {
        loop {
            let data = self.fetch();
            if let None = data {
                let ten_millis = time::Duration::from_millis(100);
                thread::sleep(ten_millis);
                continue;
            }
            let bytes = to_vec(&data).unwrap();
            self.publish(bytes);
        }
    }
    fn stop(&self) {}
}

impl<'a> Publishing<'a> for MqttPublisher {
    fn new(url: String, id: String, topics: HashMap<String, String>) -> MqttPublisher {
        // Define the set of options for the create.
        // Use an ID for a persistent session.
        let create_opts = mqtt::CreateOptionsBuilder::new()
            .server_uri(&url)
            .client_id(&id)
            .finalize();

        // Create a client.
        let client = match mqtt::Client::new(create_opts) {
            Ok(m) => m,
            Err(err) => panic!("Error creating the client: {err}"),
        };

        // Define the set of options for the connection.
        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(std::time::Duration::from_secs(20))
            .clean_session(true)
            .finalize();

        // Connect and wait for it to complete or fail.
        if let Err(e) = client.connect(conn_opts) {
            error!("Unable to connect:\n\t{:?}", e);
        } else {
            info!("Successfully connected client to host {:?}", &url);
        }

        MqttPublisher {
            topics: topics,
            client: client,
            buffer: SegQueue::new(),
            qos: 1,
        }
    }

    fn append(&self, value: Box<dyn Serialize>) {
        self.buffer.push(value);
    }

    fn fetch(&self) -> Option<Box<dyn Serialize>> {
        self.buffer.pop()
    }

    fn publish(&self, content: Vec<u8>) {
        let msg = mqtt::Message::new(self.topics.get("out").unwrap(), content, self.qos);
        let tok = self.client.publish(msg);
        if let Err(e) = tok {
            error!("Error sending message: {:?}", e);
        }
    }
}
