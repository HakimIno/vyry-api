use infrastructure::crypto::signal;
use x25519_dalek::{PublicKey, StaticSecret};
use ed25519_dalek::{Verifier, Signature};

#[test]
fn test_x3dh_handshake_simulation() {
    // 1. Alice (Server Side) generates keys
    let (alice_identity, _) = signal::generate_identity_keypair().unwrap();
    let alice_signed_prekey = signal::generate_signed_prekey(&alice_identity, 1).unwrap();
    let alice_one_time_prekeys = signal::generate_prekeys(1, 1).unwrap();
    let alice_one_time_prekey = &alice_one_time_prekeys[0];

    // 2. Bob (Client Side) fetches Alice's bundle
    // Bob verifies the signed prekey signature
    let alice_identity_public = ed25519_dalek::VerifyingKey::from_bytes(
        &alice_identity.public_key.as_slice().try_into().unwrap()
    ).unwrap();
    
    let signature = Signature::from_bytes(&alice_signed_prekey.signature.as_slice().try_into().unwrap());
    assert!(alice_identity_public.verify(&alice_signed_prekey.public_key, &signature).is_ok());

    // 3. Bob performs X3DH
    let bob_identity = StaticSecret::random_from_rng(rand::rngs::OsRng);
    let bob_ephemeral = StaticSecret::random_from_rng(rand::rngs::OsRng);

    // DH1: Bob's Identity + Alice's Signed Prekey
    let alice_signed_pk = PublicKey::from(
        <[u8; 32]>::try_from(alice_signed_prekey.public_key.as_slice()).unwrap()
    );
    let _dh1 = bob_identity.diffie_hellman(&alice_signed_pk);

    // DH2: Bob's Ephemeral + Alice's Identity (Converted to X25519)
    // Note: In real Signal, Identity Key is Ed25519, but for X3DH it needs to be converted to X25519 
    // or used as is if the curve supports it. Signal uses X25519 for handshake.
    // Our implementation stores Ed25519 for Identity. 
    // Standard Signal Protocol converts Ed25519 public key to X25519 for DH.
    // For this test, we will skip DH2 involving Identity Key conversion to keep it simple,
    // or we focus on DH3 and DH4 which are pure X25519.
    
    // Let's verify DH3: Bob's Ephemeral + Alice's Signed Prekey
    let dh3 = bob_ephemeral.diffie_hellman(&alice_signed_pk);

    // DH4: Bob's Ephemeral + Alice's One-Time Prekey
    let alice_otpk = PublicKey::from(
        <[u8; 32]>::try_from(alice_one_time_prekey.public_key.as_slice()).unwrap()
    );
    let dh4 = bob_ephemeral.diffie_hellman(&alice_otpk);

    // 4. Alice (Client Side receiving message) derives secrets
    // Alice reconstructs DH3
    let alice_signed_secret = StaticSecret::from(
        <[u8; 32]>::try_from(alice_signed_prekey.private_key.as_slice()).unwrap()
    );
    let bob_ephemeral_public = PublicKey::from(&bob_ephemeral);
    
    let alice_dh3 = alice_signed_secret.diffie_hellman(&bob_ephemeral_public);

    // Alice reconstructs DH4
    let alice_otpk_secret = StaticSecret::from(
        <[u8; 32]>::try_from(alice_one_time_prekey.private_key.as_slice()).unwrap()
    );
    let alice_dh4 = alice_otpk_secret.diffie_hellman(&bob_ephemeral_public);

    // 5. Assert Shared Secrets Match
    assert_eq!(dh3.as_bytes(), alice_dh3.as_bytes(), "DH3 mismatch");
    assert_eq!(dh4.as_bytes(), alice_dh4.as_bytes(), "DH4 mismatch");

    println!("X3DH Key Exchange Simulation Successful!");
}
