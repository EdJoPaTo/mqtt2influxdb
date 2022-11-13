use std::time::{Duration, SystemTime};

use rumqttc::{AsyncClient, MqttOptions, QoS};
use tokio::sync::mpsc::{channel, Receiver};
use tokio::task;
use tokio::time::sleep;

use crate::message::Message;

pub async fn connect(
    broker: &str,
    port: u16,
    username: Option<&str>,
    password: Option<&str>,
    topics: Vec<String>,
    verbose: bool,
) -> (AsyncClient, Receiver<Message>) {
    let client_id = format!("mqtt2influxdb-{:x}", rand::random::<u32>());
    let mut mqttoptions = MqttOptions::new(client_id, broker, port);

    if let Some(password) = password {
        let username = username.unwrap();
        mqttoptions.set_credentials(username, password);
    }

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 100);

    loop {
        let event = eventloop.poll().await.expect("MQTT connection error");
        if let rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_)) = event {
            subscribe(&client, topics.clone())
                .await
                .expect("failed to subscribe to MQTT topic");
            break;
        }
    }

    let (sender, receiver) = channel(100);

    let resultclient = client.clone();
    task::spawn(async move {
        loop {
            let event = eventloop.poll().await;
            if verbose {
                println!("MQTT Event {event:?}");
            }
            match event {
                Ok(rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(p))) => {
                    println!("MQTT connected {p:?}");
                    if !p.session_present {
                        subscribe(&client, topics.clone())
                            .await
                            .expect("failed to subscribe after reconnect");
                    }
                }
                Ok(rumqttc::Event::Outgoing(rumqttc::Outgoing::Disconnect)) => {
                    println!("MQTT Disconnect happening...");
                    break;
                }
                Ok(rumqttc::Event::Incoming(rumqttc::Packet::Publish(p)))
                    if !p.dup && !p.retain && !p.payload.is_empty() =>
                {
                    let nanos = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos();
                    let message = Message::new(nanos, p.topic, p.payload.to_vec());
                    sender.send(message).await.expect("receiver died");
                }
                Ok(_) => {}
                Err(err) => {
                    println!("MQTT Connection Error: {err}");
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
    });

    (resultclient, receiver)
}

async fn subscribe(client: &AsyncClient, topics: Vec<String>) -> Result<(), rumqttc::ClientError> {
    for topic in topics {
        client.subscribe(topic, QoS::ExactlyOnce).await?;
    }
    Ok(())
}
