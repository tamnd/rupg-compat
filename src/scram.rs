//! The client side of SCRAM-SHA-256 (RFC 5802 and RFC 7677), for replay.
//!
//! A SCRAM exchange has random nonces, so replay cannot send the exchange of a trace again. It authenticates again with each server (spec/21 section 21.3.5). The harness has few dependencies, so SHA-256, HMAC, PBKDF2 and base64 are here. They are the plain algorithms of FIPS 180-4, RFC 2104, RFC 8018 and RFC 4648.

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

const H0: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// The SHA-256 of `data`.
pub(crate) fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h = H0;
    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut tail = data[data.len() - data.len() % 64..].to_vec();
    tail.push(0x80);
    while tail.len() % 64 != 56 {
        tail.push(0);
    }
    tail.extend_from_slice(&bit_len.to_be_bytes());
    for block in data.as_chunks::<64>().0.iter().chain(tail.as_chunks::<64>().0) {
        compress(&mut h, block);
    }
    let mut out = [0u8; 32];
    for (chunk, word) in out.as_chunks_mut::<4>().0.iter_mut().zip(h) {
        *chunk = word.to_be_bytes();
    }
    out
}

fn compress(h: &mut [u32; 8], block: &[u8; 64]) {
    let mut w = [0u32; 64];
    for (i, word) in block.as_chunks::<4>().0.iter().enumerate() {
        w[i] = u32::from_be_bytes(*word);
    }
    for i in 16..64 {
        let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
        let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
        w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
    }
    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = *h;
    for i in 0..64 {
        let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let ch = (e & f) ^ (!e & g);
        let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
        let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let maj = (a & b) ^ (a & c) ^ (b & c);
        let t2 = s0.wrapping_add(maj);
        hh = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }
    for (x, y) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
        *x = x.wrapping_add(y);
    }
}

/// HMAC-SHA-256 (RFC 2104).
pub(crate) fn hmac(key: &[u8], data: &[u8]) -> [u8; 32] {
    let mut k = [0u8; 64];
    if key.len() > 64 {
        k[..32].copy_from_slice(&sha256(key));
    } else {
        k[..key.len()].copy_from_slice(key);
    }
    let mut inner: Vec<u8> = k.iter().map(|b| b ^ 0x36).collect();
    inner.extend_from_slice(data);
    let mut outer: Vec<u8> = k.iter().map(|b| b ^ 0x5c).collect();
    outer.extend_from_slice(&sha256(&inner));
    sha256(&outer)
}

/// PBKDF2 with HMAC-SHA-256 and one block of output, the `Hi` function of SCRAM.
pub(crate) fn hi(password: &[u8], salt: &[u8], iterations: u32) -> [u8; 32] {
    let mut s = salt.to_vec();
    s.extend_from_slice(&1u32.to_be_bytes());
    let mut u = hmac(password, &s);
    let mut out = u;
    for _ in 1..iterations {
        u = hmac(password, &u);
        for (o, x) in out.iter_mut().zip(u) {
            *o ^= x;
        }
    }
    out
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub(crate) fn base64(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let n = chunk.iter().enumerate().fold(0u32, |n, (i, b)| n | u32::from(*b) << (16 - 8 * i));
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(char::from(B64[(n >> (18 - 6 * i) & 63) as usize]));
            } else {
                out.push('=');
            }
        }
    }
    out
}

pub(crate) fn unbase64(s: &str) -> Option<Vec<u8>> {
    let s = s.trim_end_matches('=');
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let (mut n, mut bits) = (0u32, 0);
    for c in s.bytes() {
        let v = B64.iter().position(|&b| b == c)? as u32;
        n = n << 6 | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((n >> bits) as u8);
        }
    }
    Some(out)
}

/// Random bytes for a nonce, from the operating system when it has `/dev/urandom`.
pub(crate) fn random_bytes(n: usize) -> Vec<u8> {
    use std::io::Read as _;
    let mut out = vec![0u8; n];
    if std::fs::File::open("/dev/urandom").and_then(|mut f| f.read_exact(&mut out)).is_ok() {
        return out;
    }
    use std::hash::{BuildHasher as _, Hasher as _};
    let state = std::collections::hash_map::RandomState::new();
    for (i, b) in out.iter_mut().enumerate() {
        let mut h = state.build_hasher();
        h.write_usize(i);
        h.write_u128(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0),
        );
        *b = h.finish() as u8;
    }
    out
}

/// One SCRAM-SHA-256 exchange from the client side, without channel binding.
#[derive(Debug)]
pub(crate) struct Client {
    password: Vec<u8>,
    first_bare: String,
    nonce: String,
    server_signature: Option<[u8; 32]>,
}

impl Client {
    /// Starts an exchange. PostgreSQL ignores the user name in SCRAM, so libpq sends an empty one, and so does this client.
    pub(crate) fn new(password: &str, nonce: Option<&str>) -> Client {
        let nonce = nonce.map(str::to_string).unwrap_or_else(|| base64(&random_bytes(18)));
        Client::with_user("", password, &nonce)
    }

    fn with_user(user: &str, password: &str, nonce: &str) -> Client {
        Client {
            password: password.as_bytes().to_vec(),
            first_bare: format!("n={user},r={nonce}"),
            nonce: nonce.to_string(),
            server_signature: None,
        }
    }

    /// The client-first message: the body of `SASLInitialResponse`.
    pub(crate) fn first(&self) -> String {
        format!("n,,{}", self.first_bare)
    }

    /// Takes the server-first message (`AuthenticationSASLContinue`) and gives the client-final message (`SASLResponse`).
    pub(crate) fn last(&mut self, server_first: &str) -> Result<String, String> {
        let mut nonce = None;
        let mut salt = None;
        let mut iterations = None;
        for part in server_first.split(',') {
            match part.split_at_checked(2) {
                Some(("r=", v)) => nonce = Some(v),
                Some(("s=", v)) => salt = unbase64(v),
                Some(("i=", v)) => iterations = v.parse::<u32>().ok(),
                _ => {}
            }
        }
        let (Some(nonce), Some(salt), Some(iterations)) = (nonce, salt, iterations) else {
            return Err(format!("bad server-first message {server_first:?}"));
        };
        if !nonce.starts_with(&self.nonce) || iterations == 0 {
            return Err("the server nonce does not extend the client nonce".into());
        }
        let without_proof = format!("c=biws,r={nonce}");
        let auth = format!("{},{server_first},{without_proof}", self.first_bare);
        let salted = hi(&self.password, &salt, iterations);
        let client_key = hmac(&salted, b"Client Key");
        let signature = hmac(&sha256(&client_key), auth.as_bytes());
        let proof: Vec<u8> = client_key.iter().zip(signature).map(|(k, s)| k ^ s).collect();
        self.server_signature = Some(hmac(&hmac(&salted, b"Server Key"), auth.as_bytes()));
        Ok(format!("{without_proof},p={}", base64(&proof)))
    }

    /// Checks the server-final message (`AuthenticationSASLFinal`).
    pub(crate) fn verify(&self, server_final: &str) -> Result<(), String> {
        let got = server_final.strip_prefix("v=").and_then(unbase64);
        match (got, self.server_signature) {
            (Some(g), Some(want)) if g == want => Ok(()),
            _ => Err("the server signature does not match".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(b: &[u8]) -> String {
        b.iter().map(|b| format!("{b:02x}")).collect()
    }

    #[test]
    fn sha256_known_answers() {
        assert_eq!(
            hex(&sha256(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hex(&sha256(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            hex(&sha256(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq")),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
        assert_eq!(
            hex(&sha256(&[0u8; 55])),
            "02779466cdec163811d078815c633f21901413081449002f24aa3e80f0b88ef7"
        );
        assert_eq!(
            hex(&sha256(&[0u8; 64])),
            "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
        );
    }

    #[test]
    fn hmac_rfc_4231_case_2() {
        assert_eq!(
            hex(&hmac(b"Jefe", b"what do ya want for nothing?")),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
    }

    #[test]
    fn pbkdf2_known_answer() {
        // RFC 7914 section 11, PBKDF2-HMAC-SHA256 with c = 1, the first 32 bytes.
        assert_eq!(
            hex(&hi(b"passwd", b"salt", 1)),
            "55ac046e56e3089fec1691c22544b605f94185216dde0465e68b9d57c20dacbc"
        );
    }

    #[test]
    fn base64_round_trips() {
        for (raw, enc) in
            [("", ""), ("f", "Zg=="), ("ab", "YWI="), ("foo", "Zm9v"), ("foobar", "Zm9vYmFy")]
        {
            assert_eq!(base64(raw.as_bytes()), enc);
            assert_eq!(unbase64(enc).unwrap(), raw.as_bytes());
        }
        assert_eq!(unbase64("biws").unwrap(), b"n,,");
        assert!(unbase64("a!b").is_none());
    }

    #[test]
    fn the_exchange_of_rfc_7677() {
        let mut c = Client::with_user("user", "pencil", "rOprNGfwEbeRWgbNEkqO");
        assert_eq!(c.first(), "n,,n=user,r=rOprNGfwEbeRWgbNEkqO");
        let last = c
            .last("r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096")
            .unwrap();
        assert_eq!(
            last,
            "c=biws,r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,p=dHzbZapWIk4jUhN+Ute9ytag9zjfMHgsqmmiz7AndVQ="
        );
        assert!(c.verify("v=6rriTRBi23WpRR/wtup+mMhUZUn/dB5nLTJRsjl95G4=").is_ok());
        assert!(c.verify("v=AAAA").is_err());
    }

    #[test]
    fn a_nonce_that_does_not_extend_ours_is_an_error() {
        let mut c = Client::new("pw", Some("abc"));
        assert!(c.last("r=xyz,s=AAAA,i=4096").is_err());
        assert!(c.last("r=abc1").is_err());
    }
}
