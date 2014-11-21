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

use command::Command;
use self::TransactionState::{Unconnected, Connected, HelloSent, MailSent, RecipientSent, DataSent};

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
            (Unconnected, &Command::Connect) => true,
            (Unconnected, _) => false,
            // Only a message can follow the DATA command
            (DataSent, &Command::Message) => true,
            (DataSent, _) => false,
            // Commands that can be issued everytime
            (_, &Command::ExtendedHello(_)) => true,
            (_, &Command::Hello(_)) => true,
            (_, &Command::Reset) => true,
            (_, &Command::Verify(_)) => true,
            (_, &Command::Expand(_)) => true,
            (_, &Command::Help(_)) => true,
            (_, &Command::Noop) => true,
            (_, &Command::Quit) => true,
            // Commands that require a particular state
            (HelloSent, &Command::Mail(_, _)) => true,
            (MailSent, &Command::Recipient(_, _)) => true,
            (RecipientSent, &Command::Recipient(_, _)) => true,
            (RecipientSent, &Command::Data) => true,
            // Everything else
            (_, _) => false,
        }
    }

    /// Returns the state resulting of the given command
    ///
    /// A `None` return value means the comand is not allowed.
    pub fn next_state(&mut self, command: &Command) -> Option<TransactionState> {
        match (*self, command) {
            (Unconnected, &Command::Connect) => Some(Connected),
            (Unconnected, _) => None,
            (DataSent, &Command::Message) => Some(HelloSent),
            (DataSent, _) => None,
            // Commands that can be issued everytime
            (_, &Command::ExtendedHello(_)) => Some(HelloSent),
            (_, &Command::Hello(_)) => Some(HelloSent),
            (Connected, &Command::Reset) => Some(Connected),
            (_, &Command::Reset) => Some(HelloSent),
            (state, &Command::Verify(_)) => Some(state),
            (state, &Command::Expand(_)) => Some(state),
            (state, &Command::Help(_)) => Some(state),
            (state, &Command::Noop) => Some(state),
            (_, &Command::Quit) => Some(Unconnected),
            // Commands that require a particular state
            (HelloSent, &Command::Mail(_, _)) => Some(MailSent),
            (MailSent, &Command::Recipient(_, _)) => Some(RecipientSent),
            (RecipientSent, &Command::Recipient(_, _)) => Some(RecipientSent),
            (RecipientSent, &Command::Data) => Some(DataSent),
            // Everything else
            (_, _) => None,
        }
    }
}

#[cfg(test)]
mod test {
    use command::Command;
    use super::TransactionState;

    #[test]
    fn test_new() {
        assert_eq!(TransactionState::new(), TransactionState::Unconnected);
    }

    #[test]
    fn test_is_command_allowed() {
        assert!(!TransactionState::Unconnected.is_command_allowed(&Command::Noop));
        assert!(!TransactionState::DataSent.is_command_allowed(&Command::Noop));
        assert!(TransactionState::HelloSent.is_command_allowed(&Command::Mail("".to_string(), None)));
        assert!(!TransactionState::MailSent.is_command_allowed(&Command::Mail("".to_string(), None)));
    }

    #[test]
    fn test_next_state() {
        assert_eq!(TransactionState::MailSent.next_state(&Command::Noop), Some(TransactionState::MailSent));
        assert_eq!(TransactionState::HelloSent.next_state(&Command::Mail("".to_string(), None)),
                   Some(TransactionState::MailSent));
        assert_eq!(TransactionState::MailSent.next_state(&Command::Mail("".to_string(), None)), None);
    }
}
