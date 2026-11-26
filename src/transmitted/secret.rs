use crate::err::Error;
use ring::error::Unspecified;
use ring::rand::{self, SystemRandom};
use ring::{digest, pbkdf2};
use std::io;
use std::num::NonZeroU32;

const PBKDF2_ALG: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA256;
const PBKDF2_LEN: usize = digest::SHA256_OUTPUT_LEN;
const PBKDF2_ITERATIONS: NonZeroU32 = NonZeroU32::new(100_000).unwrap();
const SALT_LEN: usize = 16;

pub struct Key {
    salt: [u8; SALT_LEN],
    hash: [u8; PBKDF2_LEN],
}

impl Key {
    pub fn new(secret_key: &[u8]) -> Result<Self, Error> {
        let salt = rand::generate::<[u8; SALT_LEN]>(&SystemRandom::new())
            .map_err(|Unspecified| io::Error::other("RNG initialization failed"))?
            .expose();

        let mut hash = [0u8; PBKDF2_LEN];
        pbkdf2::derive(PBKDF2_ALG, PBKDF2_ITERATIONS, &salt, secret_key, &mut hash);

        Ok(Self { salt, hash })
    }

    pub fn verify(&self, secret_key: &[u8]) -> Result<(), ()> {
        pbkdf2::verify(
            PBKDF2_ALG,
            PBKDF2_ITERATIONS,
            &self.salt,
            secret_key,
            &self.hash,
        )
        .map_err(|Unspecified| ())
    }
}
