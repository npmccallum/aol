use std::time::Instant;

use human_size::{Byte, Megabyte, SpecificSize};
use p256::ecdsa::signature::{Signature as _, Signer, Verifier};
use p256::ecdsa::{Signature, SigningKey};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sha2::Digest;

#[derive(Serialize, Deserialize)]
struct Message {
    kind: String,

    #[serde(with = "serde_bytes")]
    prev: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize)]
struct Envelope {
    #[serde(with = "serde_bytes")]
    message: Vec<u8>,

    #[serde(with = "serde_bytes")]
    signature: Vec<u8>,
}

fn main() {
    let mut args = std::env::args();
    args.next().unwrap(); // skip

    let entries: usize = args.next().unwrap().parse().unwrap();
    let nosig = args.next().as_deref() == Some("nosig");

    let key = SigningKey::random(rand::thread_rng());

    // Create an append only log.
    let mut aol = Vec::new();
    let start = Instant::now();
    for _ in 0..entries {
        // Create a random kind string for some entropy.
        let kind = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .collect();

        // Create a message.
        let msg = Message {
            kind: String::from_utf8(kind).unwrap(),
            prev: aol.last().map(|prev| {
                let mut env = Vec::new();
                ciborium::ser::into_writer(prev, &mut env).unwrap();
                sha2::Sha256::digest(env).to_vec()
            }),
        };

        // Serialize the message.
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&msg, &mut buf).unwrap();

        // Sign the message.
        let env = Envelope {
            signature: key.sign(&buf).to_vec(),
            message: buf,
        };

        aol.push(env);
    }
    let end = Instant::now();

    let mut log = Vec::new();
    ciborium::ser::into_writer(&aol, &mut log).unwrap();

    eprintln!("entries: {}", entries);
    eprintln!("time: {:?}", end - start);
    eprintln!(
        "size: {}",
        SpecificSize::new(log.len() as f64, Byte)
            .unwrap()
            .into::<Megabyte>()
    );

    let start = Instant::now();
    let mut prev = None;
    for env in aol {
        // Verify the signature.
        if !nosig {
            let sig = Signature::from_bytes(&env.signature).unwrap();
            key.verifying_key().verify(&env.message, &sig).unwrap();
        }

        // Verify the previous hash.
        let msg: Message = ciborium::de::from_reader(&env.message[..]).unwrap();
        assert_eq!(prev, msg.prev);

        // Hash the entry for the next validation.
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&env, &mut buf).unwrap();
        prev = Some(sha2::Sha256::digest(buf).to_vec());
    }
    let end = Instant::now();

    eprintln!("sig: {}", !nosig);
    eprintln!("time: {:?}", end - start);
    eprintln!("byte: {:?}", prev.as_ref().and_then(|x| x.first()));
}
