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
//

pub fn hash_password(argon2_hash: &str, salt: PasswordSalt) -> ApiResult<String> {
    let argon2_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(argon2_hash)
        .map_err(|err| ApiError::from(err.to_string()))?;

    // although the hashed bytes contains the salt
    // concat it to the final output so we can recover it later
    let hash = |salt: &[u8]| -> Vec<u8> {
        let msg = [salt, &argon2_bytes].concat();
        let sha256_hash = Sha256::digest(msg);
        [salt, &sha256_hash].concat()
    };

    let password_hash = match salt {
        PasswordSalt::CreateNew => hash(&OsRng.gen::<[u8; 32]>()),
        PasswordSalt::Recover { password_hash } => {
            let password_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
                .decode(password_hash)
                .map_err(|err| ApiError::from(err.to_string()))?;
            hash(&password_bytes[..32])
            // ignore the rest of the password_bytes... will be checked for comparison in the caller
        }
    };

    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&password_hash))
}

// The salt used for the hash to ensure some randomness
pub enum PasswordSalt<'a> {
    // when creating a new salt, we just generate a random one
    CreateNew,
    // when recovering a salt from an existing hash, we extract it
    // from the first 32 bytes of the password hash
    Recover { password_hash: &'a str },
}
