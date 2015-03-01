// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Authentication mecanisms

use common::NUL;
use client::authentication::AuthenticationMecanism;

struct Plain {
    identity: String,
    username: String,
    password: String,
}

impl AuthenticationMecanism for Plain {
    fn new(username: String, password: String) -> Plain {
        Plain {
            identity: "".to_string(),
            username: username,
            password: password,
        }
    }

    fn initial_response(&self) -> Option<String> {
        Some(self.response(""))
    }

    fn response(&self, challenge: &str) -> String {
        // We do not need a challenge in PLAIN authentication
        let _ = challenge;
        format!("{}{}{}{}{}", self.identity, NUL, self.username, NUL, self.password)
    }
}
