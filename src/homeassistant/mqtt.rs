use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde::Serialize;

pub(crate) struct HomeAssistantMQTTClient {
    client: AsyncClient,
    eventloop: rumqttc::EventLoop,
}

impl HomeAssistantMQTTClient {
    pub(crate) fn new(mqtt_options: MqttOptions) -> Self {
        let (client, eventloop) = AsyncClient::new(mqtt_options, 10);
        Self {
            client,
            eventloop,
        }
    }

    async fn wait_ack(&mut self) -> anyhow::Result<()> {
        let _ = self.eventloop.poll().await.unwrap();
        let _ = self.eventloop.poll().await.unwrap();
        Ok(())
    }

    pub async fn publish_and_wait(&mut self, topic: String, data: impl Serialize) -> anyhow::Result<()> {
        let serialized = serde_json::to_string(&data)?;
        self.client.publish(topic, QoS::AtLeastOnce, false, serialized).await?;
        self.wait_ack().await
    }

    pub async fn disconnect(&mut self) -> anyhow::Result<()> {
        self.client.disconnect().await?;
        Ok(())
    }
}
