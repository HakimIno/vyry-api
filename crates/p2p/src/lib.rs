use anyhow::Result;
use bytes::Bytes;
use std::sync::Arc;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_init::RTCDataChannelInit;
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;

// Constants for tuning performance
const CHUNK_SIZE: usize = 64 * 1024; // 64KB chunks
const BUFFER_THRESHOLD: usize = 1024 * 1024; // 1MB buffer

pub struct P2PClient {
    pub peer_connection: Arc<RTCPeerConnection>,
    pub data_channel: Option<Arc<RTCDataChannel>>,
}

impl P2PClient {
    pub async fn new() -> Result<Self> {
        let mut m = MediaEngine::default();
        m.register_default_codecs()?;

        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut m)?;

        let api = APIBuilder::new()
            .with_media_engine(m)
            .with_interceptor_registry(registry)
            .build();

        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };

        let peer_connection = Arc::new(api.new_peer_connection(config).await?);

        // Handle incoming data channels (for the recipient/Bob)
        peer_connection.on_data_channel(Box::new(move |d: Arc<RTCDataChannel>| {
            let d_label = d.label().to_owned();
            let d_id = d.id();
            tracing::info!("New DataChannel {} {}", d_label, d_id);

            // Register on_message for this channel too
            d.on_message(Box::new(move |msg: webrtc::data_channel::data_channel_message::DataChannelMessage| {
                let payload = msg.data;
                tracing::info!("Received {} bytes on '{}'", payload.len(), d_label);
                Box::pin(async {})
            }));
            
            Box::pin(async {})
        }));

        Ok(Self {
            peer_connection,
            data_channel: None,
        })
    }

    pub async fn create_offer(&self) -> Result<RTCSessionDescription> {
        let offer = self.peer_connection.create_offer(None).await?;
        let mut gather_complete = self.peer_connection.gathering_complete_promise().await;
        self.peer_connection.set_local_description(offer).await?;
        let _ = gather_complete.recv().await;

        if let Some(local_desc) = self.peer_connection.local_description().await {
            Ok(local_desc)
        } else {
            Err(anyhow::anyhow!("Failed to generate local description"))
        }
    }

    pub async fn create_answer(&self, offer_sdp: String) -> Result<RTCSessionDescription> {
        let desc = RTCSessionDescription::offer(offer_sdp)?;
        self.peer_connection.set_remote_description(desc).await?;

        let answer = self.peer_connection.create_answer(None).await?;
        let mut gather_complete = self.peer_connection.gathering_complete_promise().await;
        self.peer_connection.set_local_description(answer).await?;
        let _ = gather_complete.recv().await;

        if let Some(local_desc) = self.peer_connection.local_description().await {
            Ok(local_desc)
        } else {
            Err(anyhow::anyhow!("Failed to generate local description"))
        }
    }

    pub async fn set_remote_offer(&self, offer_sdp: String) -> Result<()> {
        let desc = RTCSessionDescription::offer(offer_sdp)?;
        self.peer_connection.set_remote_description(desc).await?;
        Ok(())
    }

    pub async fn set_remote_answer(&self, answer_sdp: String) -> Result<()> {
        let desc = RTCSessionDescription::answer(answer_sdp)?;
        self.peer_connection.set_remote_description(desc).await?;
        Ok(())
    }

    pub async fn create_data_channel(&mut self, label: &str) -> Result<Arc<RTCDataChannel>> {
        let ordered = true;
        let _max_retransmits = 0; // Unreliable mode for speed? No, file transfer needs reliability.
        // For max speed, we use ordered=true but we can tune buffer.
        
        let options = RTCDataChannelInit {
            ordered: Some(ordered),
            // max_retransmits: Some(max_retransmits), 
            ..Default::default()
        };

        let data_channel = self.peer_connection.create_data_channel(label, Some(options)).await?;
        
        // Set up callbacks
        let dc_clone = data_channel.clone();
        data_channel.on_open(Box::new(move || {
            Box::pin(async move {
                tracing::info!("Data channel '{}' opened", dc_clone.label());
            })
        }));

        let dc_clone_2 = data_channel.clone();
        data_channel.on_message(Box::new(move |msg: webrtc::data_channel::data_channel_message::DataChannelMessage| {
            let dc = dc_clone_2.clone();
            Box::pin(async move {
                let payload = msg.data;
                tracing::info!("Received {} bytes on '{}'", payload.len(), dc.label());
                // In a real app, we would send this to a callback or channel.
                // For now, we just log it. 
                // TODO: Add a way to extract this data for the application/test.
            })
        }));

        self.data_channel = Some(data_channel.clone());
        Ok(data_channel)
    }

    pub async fn send_file(&self, data: Vec<u8>) -> Result<()> {
        if let Some(dc) = &self.data_channel {
            // Chunking logic
            for chunk in data.chunks(CHUNK_SIZE) {
                // Backpressure handling: check buffered amount
                while dc.buffered_amount().await > BUFFER_THRESHOLD {
                    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                }
                dc.send(&Bytes::copy_from_slice(chunk)).await?;
            }
            tracing::info!("File sent successfully: {} bytes", data.len());
            Ok(())
        } else {
            Err(anyhow::anyhow!("No data channel"))
        }
    }
}
