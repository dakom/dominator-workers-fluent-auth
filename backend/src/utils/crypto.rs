use base64::Engine;
use rand::{prelude::*, rngs::OsRng};
use sha2::{Digest, Sha256};
use shared::backend::result::{ApiError, ApiResult};

// the password was sent as an argon2 hash from the client
// but we must hash it again, otherwise that might as well just be plaintext
// if the db is compromised (user can just send the db value for comparison)
// however, we don't need a compute-intensive hash here, since the
// plaintext isn't the original password, it's the argon2 output bytes
// and so an attacker would need to brute force sha256 guesses against the argon2 output space
pub fn hash_password(password:&str, salt: PasswordSalt) -> ApiResult<String> {
    let password = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(password).map_err(|err| ApiError::from(err.to_string()))?;
    let password = match salt {
        PasswordSalt::Recover => {
            inner(&password[..32], &password[32..])
        },
        PasswordSalt::CreateNew => {
            inner(&OsRng.gen::<[u8; 32]>(), &password)
        }
    };

    fn inner(salt: &[u8], password: &[u8]) -> Vec<u8> {
        let msg = [salt, password].concat(); 
        let hash = Sha256::digest(msg);
        let password = [salt, &hash].concat();
        password
    }

    let password = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&password);

    Ok(password)
}

// The salt used for the hash to ensure some randomness
pub enum PasswordSalt {
    // when creating a new salt, we just generate a random one
    CreateNew,
    // when recovering a salt from an existing hash, we extract it
    // from the first 32 bytes of the hash
    Recover
}