//! An implementation of the [SHA-1][1] cryptographic hash algorithm.
//!
//! # Usage
//!
//! ```rust
//! use hex_literal::hex;
//! use sha1::{Sha1, Digest};
//!
//! // create a Sha1 object
//! let mut hasher = Sha1::new();
//!
//! // process input message
//! hasher.update(b"hello world");
//!
//! // acquire hash digest in the form of GenericArray,
//! // which in this case is equivalent to [u8; 20]
//! let result = hasher.finalize();
//! assert_eq!(result[..], hex!("2aae6c35c94fcfb415dbe95f408b9ce91ee846ed"));
//! ```
//!
//! Also see [RustCrypto/hashes][2] readme.
//!
//! [1]: https://en.wikipedia.org/wiki/SHA-1
//! [2]: https://github.com/RustCrypto/hashes

#![no_std]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg"
)]
#![deny(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

pub use digest::{self, Digest};

use core::{fmt, slice::from_ref};
use digest::{
    block_buffer::BlockBuffer,
    generic_array::{
        typenum::{Unsigned, U20, U64},
        GenericArray,
    },
    AlgorithmName, FixedOutputCore, UpdateCore, UpdateCoreWrapper,
};

mod compress;
mod consts;

use compress::compress;
use consts::{H, STATE_LEN};

type Block = GenericArray<u8, U64>;

/// Core SHA-1 hasher state.
#[derive(Clone)]
pub struct Sha1Core {
    h: [u32; STATE_LEN],
    block_len: u64,
}

impl UpdateCore for Sha1Core {
    type BlockSize = U64;

    fn update_blocks(&mut self, blocks: &[Block]) {
        self.block_len += blocks.len() as u64;
        compress(&mut self.h, blocks);
    }
}

impl FixedOutputCore for Sha1Core {
    type OutputSize = U20;

    #[inline]
    fn finalize_fixed_core(
        &mut self,
        buffer: &mut BlockBuffer<Self::BlockSize>,
        out: &mut GenericArray<u8, Self::OutputSize>,
    ) {
        let bs = Self::BlockSize::U64;
        let bit_len = 8 * (buffer.get_pos() as u64 + bs * self.block_len);

        let mut h = self.h;
        buffer.len64_padding_be(bit_len, |b| compress(&mut h, from_ref(b)));
        for (chunk, v) in out.chunks_exact_mut(4).zip(h.iter()) {
            chunk.copy_from_slice(&v.to_be_bytes());
        }
    }
}

impl Default for Sha1Core {
    #[inline]
    fn default() -> Self {
        Self { h: H, block_len: 0 }
    }
}

impl AlgorithmName for Sha1Core {
    const NAME: &'static str = "Sha1";
}

impl fmt::Debug for Sha1Core {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Sha1Core { ... }")
    }
}

/// SHA-1 hasher state.
pub type Sha1 = UpdateCoreWrapper<Sha1Core>;
