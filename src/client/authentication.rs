// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Provides authentication functions

use serialize::base64::{self, ToBase64, FromBase64};
use serialize::hex::ToHex;
use crypto::hmac::Hmac;
use crypto::md5::Md5;
use crypto::mac::Mac;

use NUL;
use error::Error;

/// Returns a PLAIN mecanism response
pub fn plain(username: &str, password: &str) -> String {
    format!("{}{}{}{}", NUL, username, NUL, password).as_bytes().to_base64(base64::STANDARD)
}

/// Returns a CRAM-MD5 mecanism response
pub fn cram_md5(username: &str, password: &str, encoded_challenge: &str) -> Result<String, Error> {
    let challenge = match encoded_challenge.from_base64() {
        Ok(challenge) => challenge,
        Err(error) => return Err(Error::ChallengeParsingError(error)),
    };

    let mut hmac = Hmac::new(Md5::new(), password.as_bytes());
    hmac.input(&challenge);

    Ok(format!("{} {}", username, hmac.result().code().to_hex()).as_bytes().to_base64(base64::STANDARD))
}

#[cfg(test)]
mod test {
    use super::{plain, cram_md5};

    #[test]
    fn test_plain() {
        assert_eq!(plain("username", "password"), "AHVzZXJuYW1lAHBhc3N3b3Jk");
    }

    #[test]
    fn test_cram_md5() {
        assert_eq!(cram_md5("alice", "wonderland",
            "PDE3ODkzLjEzMjA2NzkxMjNAdGVzc2VyYWN0LnN1c2FtLmluPg==").unwrap(),
            "YWxpY2UgNjRiMmE0M2MxZjZlZDY4MDZhOTgwOTE0ZTIzZTc1ZjA=");
    }
}
