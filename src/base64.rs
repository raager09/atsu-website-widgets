const STANDARD: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const URL_SAFE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

pub fn encode(bytes: &[u8]) -> String {
    encode_with(bytes, STANDARD, true)
}

pub fn encode_url(bytes: &[u8]) -> String {
    encode_with(bytes, URL_SAFE, false)
}

fn encode_with(bytes: &[u8], alphabet: &[u8; 64], pad: bool) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let at = |i: usize| chunk.get(i).copied().unwrap_or(0);
        let n = u32::from_be_bytes([0, at(0), at(1), at(2)]);
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(alphabet[((n >> (18 - 6 * i)) & 63) as usize] as char);
            } else if pad {
                out.push('=');
            }
        }
    }
    out
}
