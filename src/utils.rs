use polynomial_ring::Polynomial;
use std::hash::Hasher;
use rs_sha3_256::{Sha3_256Hasher, HasherContext};
use rs_sha3_512::Sha3_512Hasher;
use rs_shake128::Shake128Hasher;
use rs_shake256::Shake256Hasher;
use getrandom::getrandom;
use aes_ctr_drbg::DrbgCtx;

/// default parameters for module-LWE
pub struct Parameters {
	/// degree of the polynomials
    pub n: usize,
	/// Ciphertext modulus
    pub q: i64,
	/// Module rank	
    pub k: usize,
    /// Standard deviation of the error
    pub sigma: f64,
	/// 2n-th root of unity	
    pub omega: i64,
	/// Polynomial modulus
    pub f: Polynomial<i64>,
    /// Function to generate random bytes
    pub random_bytes: fn(usize, Option<DrbgCtx>) -> Vec<u8>,
}

/// generate random bytes using `getrandom` crate. 
fn gen_random_bytes(size: usize, drbg: Option<DrbgCtx>) -> Vec<u8> {
	let mut out = vec![0; size];
	if let Some(mut drbg) = drbg {
		drbg.get_random(&mut out);
	}
	else {
		//let mut out = vec![0u8; size];
		getrandom(&mut out).expect("Failed to get random bytes");
	}
	out
}

/// default parameters for module-LWE
impl Default for Parameters {
    fn default() -> Self {
        let n = 512;
        let q = 12289;
        let k = 8;
        let sigma = 3.19;
		let omega = ntt::omega(q, 2*n);
        let mut poly_vec = vec![0i64;n+1];
        poly_vec[0] = 1;
        poly_vec[n] = 1;
        let f = Polynomial::new(poly_vec);
        Parameters { n, q, k, sigma, omega, f, random_bytes: gen_random_bytes }
    }
}

/// Hash function described in 4.4 of FIPS 203 (page 18)
///
/// # Arguments
///
/// * `m` - A vector of 64-bit integers
///
/// # Returns
///
/// * Vec<u8> - 32 byte output, the result of applying the sha3_256 hash
///
/// # Example
/// ```
/// use ml_kem::utils::hash_h;
/// let input = vec![1i64, 2, 3, 4, 5, 6, 7, 8];
/// let result = hash_h(input);
/// assert_eq!(result.len(), 32); // Ensure the result is 32 bytes long
/// ```
pub fn hash_h(m: Vec<i64>) -> Vec<u8> {
    // Convert i64 vector directly into a byte slice
    let bytes: Vec<u8> = m.iter()
        .flat_map(|num| num.to_le_bytes()) // Convert each i64 to 8 bytes (little-endian)
        .collect();

    // Apply sha3_256 hash
    let mut sha3_256hasher = Sha3_256Hasher::default();
    sha3_256hasher.write(&bytes);
    let bytes_result = HasherContext::finish(&mut sha3_256hasher);
    
    bytes_result.as_ref().to_vec() // Return the hashed output
}

/// Hash function described in 4.5 of FIPS 203 (page 18)
///
/// # Arguments
///
/// * `m` - A vector of bytes
///
/// # Returns
///
/// * Vec<u8> - 32 byte output, the result of applying the shake_256 hash
///
/// # Example
/// ```
/// use ml_kem::utils::hash_j;
/// let input = vec![0x01, 0x02, 0x03, 0x04];
/// let result = hash_j(input);
/// assert_eq!(result.len(), 32); // Ensure the result is 32 bytes long
/// ```
pub fn hash_j(m: Vec<u8>) -> Vec<u8> {
	// Apply shake_256 hash
	let mut shake_256hasher = Shake256Hasher::<32>::default();
	shake_256hasher.write(&m);
	let bytes_result = HasherContext::finish(&mut shake_256hasher);
	bytes_result[0..].to_vec()
}

/// Hash function described in 4.4 of FIPS 203 (page 18)
///
/// # Arguments
///
/// * `m` - A vector of bytes
///
/// # Returns
///
/// * (Vec<u8>, Vec<u8>) - 32 byte outputs, the result of applying the sha3_512 hash
///
/// # Example
/// ```
/// use ml_kem::utils::hash_g;
/// let input = vec![0x01, 0x02, 0x03, 0x04];
/// let (output1, output2) = hash_g(input);
/// assert_eq!(output1.len(), 32); // Ensure the first part is 32 bytes long
/// assert_eq!(output2.len(), 32); // Ensure the second part is 32 bytes long
/// ```
pub fn hash_g(m: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
	// Apply sha3_512 hash
	let mut sha3_512hasher = Sha3_512Hasher::default();
	sha3_512hasher.write(&m);
	let bytes_result = HasherContext::finish(&mut sha3_512hasher);
	// Group results
	(bytes_result[..32].to_vec(), bytes_result[32..].to_vec())
}

/// Pseudorandom function described in 4.3 of FIPS 203 (page 18)
/// Uses 128 bytes for the Shake256 hash
///
/// # Arguments
///
/// * `s` - 32 bytes
/// * `b` - A single byte
///
/// # Returns
///
/// * Vec<u8> - 128 byte output, the result of applying the shake_256 hash
///
/// # Example
/// ```
/// use ml_kem::utils::prf_2;
/// let s = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22];
/// let b = 0xFF;
/// let result = prf_2(s, b);
/// assert_eq!(result.len(), 128); // Ensure the result is 128 bytes long
/// ```
pub fn prf_2(s: Vec<u8>, b: u8) -> Vec<u8> {
	// Concatenate s and b
	let mut m = s;
	m.push(b);
	// Apply shake_256 hash
	let mut shake_256hasher = Shake256Hasher::<128>::default();
	shake_256hasher.write(&m);
	let bytes_result = HasherContext::finish(&mut shake_256hasher);
	bytes_result[0..].to_vec()
}

/// Pseudorandom function described in 4.3 of FIPS 203 (page 18)
/// Uses 192 bytes for the Shake256 hash
///
/// # Arguments
///
/// * `s` - 32 bytes
/// * `b` - A single byte
///
/// # Returns
///
/// * Vec<u8> - 192 byte output, the result of applying the shake_256 hash
///
/// # Example
/// ```
/// use ml_kem::utils::prf_3;
/// let s = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22];
/// let b = 0xFF;
/// let result = prf_3(s, b);
/// assert_eq!(result.len(), 192); // Ensure the result is 192 bytes long
/// ```
pub fn prf_3(s: Vec<u8>, b: u8) -> Vec<u8> {
	// Concatenate s and b
	let mut m = s;
	m.push(b);
	// Apply shake_256 hash
	let mut shake_256hasher = Shake256Hasher::<192>::default();
	shake_256hasher.write(&m);
	let bytes_result = HasherContext::finish(&mut shake_256hasher);
	bytes_result[0..].to_vec()
}

/// eXtendable-Output Function (XOF) described in 4.9 of FIPS 203 (page 19)
/// 
/// # Arguments
///
/// * `bytes32` - A 32-byte input
/// * `i` - An 8-bit integer, domain separation parameter
/// * `j` - An 8-bit integer, domain separation parameter
///
/// # Returns
///
/// * Vec<u8> - 840 byte output, the result of applying the shake_128 hash
///
/// # Example
/// ```
/// use ml_kem::utils::xof;
/// let bytes32 = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20];
/// let i = 0x01;
/// let j = 0x02;
/// let result = xof(bytes32, i, j);
/// assert_eq!(result.len(), 840); // Ensure the result is 840 bytes long
/// ```
pub fn xof(bytes32: Vec<u8>, i: u8, j: u8) -> Vec<u8> {
	// Concatenate bytes32, i, and j
	let mut m = bytes32;
	m.push(i);
	m.push(j);
	// Apply shake_128 hash
	let mut shake_128hasher = Shake128Hasher::<840>::default();
	shake_128hasher.write(&m);
	let bytes_result = HasherContext::finish(&mut shake_128hasher);
	bytes_result[0..].to_vec()
}