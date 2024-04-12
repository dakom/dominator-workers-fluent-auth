# Sessions and Tokens

A successful *complete* authentication session requires:

1. SigninTokenId (sent via http-only cookie, to prevent XSS attacks)
2. SigninTokenKey (sent via header, to prevent CSRF attacks)
3. UserToken (to support "sign out everywhere", not sent)
4. Validated email address

These are retrieved by going through the auth flow, either openid or email+pw

UserToken (database field)
- created on registration
- never cleaned up (unless user is completely deleted from system)
- only updated on "sign out everywhere" (change password, etc.)
- uuidv7 (simple) is used for uniqueness

AuthToken (durable object) - used for both SigninToken and OobTokens (forgot password, verify email, etc.)
- creation:
    - id is unique/random (a.k.a. SigninTokenId / OobTokenId)
    - self-delete time (configurable / provided)
    - stores:
        - key: generated random data (a.k.a. SigninTokenKey / OobTokenKey)
        - user_token: (provided, a.k.a. UserToken) 
        - uid: (provided) 
        - kind: (signin, password reset, verify email, etc.)
    - returns:
        - id
        - key
- validation:
    - instantiate against id
    - compare key
    - more validation is then picked up in the caller, if needed (user-token and email-is-verified)
        - all fully-protected routes need both of these
    - decision about expirey
        - expire now: one-time tokens
        - extend for more time: signin tokens 
    - returns:
        - uid and user_token
- transmission (response to client):
    - For sign in:
        - id is set as an http-only cookie
        - key is returned over the wire as response data
    - For Oob (password reset, email verification, etc.):
        - id and key are used to construct url which is emailed
- transmission (request from client):
    - For signin: 
        - id is automatically in cookie
        - key is manually set in header (clients store in localstorage) 
    - For Oob:
        - id and key are parsed from API request
- cleans itself up after inactivity
- (optional) cleans itself up after validation (one-time use, used for Oob flows)

OpenId Token
- Similar in concept to the AuthToken, but specifically for server <--> provider oauth flow
- Will expire as needed (time or usage)
- Upon completion, a user is registered and/or logged in, and the OpenId token is no longer needed

# Hash security and DoS prevention

## Passwords

- A user creates a password via clientside Argon2 hash
    - This prevents brute-force attacks due to the computation time of argon2
    - Doing it clientside eases load on the server and prevents DoS attacks
- The salt of the hash is the user's email + constant salt
    - It must be something predictable so that logins match registration
    - Must be unique per-row
    - In theory this could be randomized with a sortof pre-registration to allocate a random salt, but, salts merely need to be unique, not secret - so uniqueness of email per-row and system constant is globally unique and is more efficient.
    - This means that if a user changes their username, passwords are invalidated
        - This isn't in the current UI, but could be easily supported by presenting it as "confirm your password to make this change" (which would be able to validate the old hash and set the new one simultaneously)
    - To prevent matching leaked hashes elsewhere where the same algorithm may be used, configure a random constant salt in [frontend config](../frontend/src/config.rs)
- It is then sha256 hashed serverside, before being stored
    - This is fast, not a DoS vector
    - Necessary so that if the DB is leaked, attacker cannot just send the exact db value to match it
    - If the DB were leaked, original passwords would need to brute force over all possible argon2 outputs
- OpenId registrations merely create a random 32 bit value for the password

## Tokens

- The Id of tokens is created via durable object uniqueness, which is intended to be secure
- However, as an extra layer, an inner Key value is randomly generated too
- The combination of both of these (Id + Key) comprises the full token - clients must know both

## User id / user-token field

- These merely need to be somewhat random and unique. Uuid v7 is used for this purpose

## Route protection

- This is defined on the route definition. See [ROUTING DOCS](./ROUTING.md) for more details