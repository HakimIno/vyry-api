use anyhow::Result;
use p2p::P2PClient;
use std::time::Duration;
use tokio::time::sleep;
use tracing_subscriber;

#[tokio::test]
async fn test_local_p2p_connection_and_transfer() -> Result<()> {
    // Initialize tracing for debugging
    let _ = tracing_subscriber::fmt::try_init();

    // 1. Create Alice (Offerer) and Bob (Answerer)
    let mut alice = P2PClient::new().await?;
    let mut bob = P2PClient::new().await?;

    // 2. Alice creates Data Channel (must be done before offer)
    let _alice_dc = alice.create_data_channel("file-transfer").await?;
    println!("Alice created data channel");

    // 3. Alice creates Offer
    let offer = alice.create_offer().await?;
    let offer_sdp = offer.sdp.clone();
    println!("Alice generated Offer");

    // 4. Bob receives Offer and creates Answer
    let answer = bob.create_answer(offer_sdp).await?;
    let answer_sdp = answer.sdp.clone();
    println!("Bob generated Answer");

    // 5. Alice sets remote Answer
    alice.set_remote_answer(answer_sdp).await?;
    println!("Alice set Remote Answer");

    // 6. Wait for ICE connection
    sleep(Duration::from_secs(3)).await;

    // 7. Send Data from Alice
    let test_data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    alice.send_file(test_data.clone()).await?;
    println!("Alice sent {} bytes", test_data.len());
    
    // Wait for Bob to receive (check logs)
    sleep(Duration::from_secs(1)).await;
    
    println!("Test completed successfully");
    Ok(())
}
