// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! PLAIN authentication mecanism

pub mod plain;

/// Trait representing an authentication mecanism
pub trait AuthenticationMecanism {
    /// Create an authentication
    fn new(username: String, password: String) -> Self;
    /// Initial response if available
    fn initial_response(&self) -> Option<String>;
    /// Response to the given challenge
    fn response(&self, challenge: &str) -> String;
}
