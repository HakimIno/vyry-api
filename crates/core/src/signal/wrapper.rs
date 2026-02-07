use anyhow::Result;
use ed25519_dalek::{Signature, Signer, SigningKey};
use rand::rngs::OsRng;
use x25519_dalek::{PublicKey, StaticSecret};

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct IdentityKeyPair {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PreKey {
    pub id: u32,
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SignedPreKey {
    pub id: u32,
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub signature: Vec<u8>,
    pub timestamp: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SignalKeys {
    pub identity_key_pair: IdentityKeyPair,
    pub registration_id: u32,
    pub signed_prekey: SignedPreKey,
    pub one_time_prekeys: Vec<PreKey>,
}

pub fn generate_identity_keypair() -> Result<IdentityKeyPair> {
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();

    Ok(IdentityKeyPair {
        public_key: verifying_key.to_bytes().to_vec(),
        private_key: signing_key.to_bytes().to_vec(),
    })
}

pub fn generate_registration_id() -> u32 {
    rand::random::<u32>() & 0x3fff
}

pub fn generate_signed_prekey(
    identity_key_pair: &IdentityKeyPair,
    signed_prekey_id: u32,
) -> Result<SignedPreKey> {
    let secret = StaticSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&secret);

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis() as u64;

    let signing_key = SigningKey::from_bytes(&identity_key_pair.private_key.as_slice().try_into()?);
    let signature: Signature = signing_key.sign(public.as_bytes());

    Ok(SignedPreKey {
        id: signed_prekey_id,
        public_key: public.to_bytes().to_vec(),
        private_key: secret.to_bytes().to_vec(),
        signature: signature.to_bytes().to_vec(),
        timestamp,
    })
}

pub fn generate_prekeys(start_id: u32, count: u32) -> Result<Vec<PreKey>> {
    let mut prekeys = Vec::with_capacity(count as usize);
    for i in 0..count {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);

        prekeys.push(PreKey {
            id: start_id + i,
            public_key: public.to_bytes().to_vec(),
            private_key: secret.to_bytes().to_vec(),
        });
    }
    Ok(prekeys)
}


#[derive(Clone, Serialize, Deserialize)]
pub struct PreKeyBundle {
    pub device_id: u32,
    pub registration_id: u32,
    pub identity_key: Vec<u8>, // Public key
    pub signed_prekey: PublicSignedPreKey,
    pub one_time_prekey: Option<PublicPreKey>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PublicPreKey {
    pub id: u32,
    pub key: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PublicSignedPreKey {
    pub id: u32,
    pub key: Vec<u8>,
    pub signature: Vec<u8>,
}

pub fn create_signal_keys() -> Result<SignalKeys> {
    let identity_key_pair = generate_identity_keypair()?;
    let registration_id = generate_registration_id();
    let signed_prekey = generate_signed_prekey(&identity_key_pair, 1)?;
    let one_time_prekeys = generate_prekeys(1, 100)?;

    Ok(SignalKeys {
        identity_key_pair,
        registration_id,
        signed_prekey,
        one_time_prekeys,
    })
}
