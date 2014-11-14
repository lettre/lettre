// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! State of an SMTP transaction

#![unstable]

use std::fmt;
use std::fmt::{Show, Formatter};

use command;
use command::Command;

/// Contains the state of the current transaction
#[deriving(PartialEq,Eq,Clone)]
pub enum TransactionState {
    /// No connection was established
    Unconnected,
    /// The connection was successful and the banner was received
    Connected,
    /// An HELO or EHLO was successful
    HelloSent,
    /// A MAIL command was successful send
    MailSent,
    /// At least one RCPT command was sucessful
    RecipientSent,
    /// A DATA command was successful
    DataSent,
}

impl Show for TransactionState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write(
            match *self {
                Unconnected => "Unconnected",
                Connected => "Connected",
                HelloSent => "HelloSent",
                MailSent => "MailSent",
                RecipientSent => "RecipientSent",
                DataSent => "DataSent",
            }.as_bytes()
        )
    }
}

impl TransactionState {
    /// Returns the initial state
    pub fn new() -> TransactionState {
        Unconnected
    }

    /// Tests if the given command is allowed in the current state
    pub fn is_command_allowed(&self, command: &Command) -> bool {
        match (*self, command) {
            (Unconnected, &command::Connect) => true,
            (Unconnected, _) => false,
            // Only a message can follow the DATA command
            (DataSent, &command::Message) => true,
            (DataSent, _) => false,
            // Commands that can be issued everytime
            (_, &command::ExtendedHello(_)) => true,
            (_, &command::Hello(_)) => true,
            (_, &command::Reset) => true,
            (_, &command::Verify(_)) => true,
            (_, &command::Expand(_)) => true,
            (_, &command::Help(_)) => true,
            (_, &command::Noop) => true,
            (_, &command::Quit) => true,
            // Commands that require a particular state
            (HelloSent, &command::Mail(_, _)) => true,
            (MailSent, &command::Recipient(_, _)) => true,
            (RecipientSent, &command::Recipient(_, _)) => true,
            (RecipientSent, &command::Data) => true,
            // Everything else
            (_, _) => false,
        }
    }

    /// Returns the state resulting of the given command
    ///
    /// A `None` return value means the comand is not allowed.
    pub fn next_state(&mut self, command: &Command) -> Option<TransactionState> {
        match (*self, command) {
            (Unconnected, &command::Connect) => Some(Connected),
            (Unconnected, _) => None,
            (DataSent, &command::Message) => Some(HelloSent),
            (DataSent, _) => None,
            // Commands that can be issued everytime
            (_, &command::ExtendedHello(_)) => Some(HelloSent),
            (_, &command::Hello(_)) => Some(HelloSent),
            (Connected, &command::Reset) => Some(Connected),
            (_, &command::Reset) => Some(HelloSent),
            (state, &command::Verify(_)) => Some(state),
            (state, &command::Expand(_)) => Some(state),
            (state, &command::Help(_)) => Some(state),
            (state, &command::Noop) => Some(state),
            (_, &command::Quit) => Some(Unconnected),
            // Commands that require a particular state
            (HelloSent, &command::Mail(_, _)) => Some(MailSent),
            (MailSent, &command::Recipient(_, _)) => Some(RecipientSent),
            (RecipientSent, &command::Recipient(_, _)) => Some(RecipientSent),
            (RecipientSent, &command::Data) => Some(DataSent),
            // Everything else
            (_, _) => None,
        }
    }
}

#[cfg(test)]
mod test {
    use command;
    use super::TransactionState;

    #[test]
    fn test_new() {
        assert_eq!(TransactionState::new(), super::Unconnected);
    }

    #[test]
    fn test_is_command_allowed() {
        assert!(!super::Unconnected.is_command_allowed(&command::Noop));
        assert!(!super::DataSent.is_command_allowed(&command::Noop));
        assert!(super::HelloSent.is_command_allowed(&command::Mail("".to_string(), None)));
        assert!(!super::MailSent.is_command_allowed(&command::Mail("".to_string(), None)));
    }

    #[test]
    fn test_next_state() {
        assert_eq!(super::MailSent.next_state(&command::Noop), Some(super::MailSent));
        assert_eq!(super::HelloSent.next_state(&command::Mail("".to_string(), None)),
                   Some(super::MailSent));
        assert_eq!(super::MailSent.next_state(&command::Mail("".to_string(), None)), None);
    }
}
