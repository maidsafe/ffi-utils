// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use base64::{self, DecodeError, URL_SAFE_NO_PAD};

/// Encode the data using base64 encoding.
pub fn base64_encode(input: &[u8]) -> String {
    base64::encode_config(input, URL_SAFE_NO_PAD)
}

/// Decode base64 encoded data.
pub fn base64_decode(input: &str) -> Result<Vec<u8>, DecodeError> {
    base64::decode_config(input, URL_SAFE_NO_PAD)
}
