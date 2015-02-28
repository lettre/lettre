// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! State of an SMTP transaction

use command::Command;
use self::TransactionState::{Unconnected, Connected, HelloSent, MailSent, RecipientSent, DataSent, AuthenticateSent};

/// Contains the state of the current transaction
#[derive(PartialEq,Eq,Copy,Debug)]
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
    /// An authenticate without initial response was successful
    AuthenticateSent,
}

impl TransactionState {
    /// Returns the initial state
    pub fn new() -> TransactionState {
        Unconnected
    }

    /// Tests if the given command is allowed in the current state
    pub fn is_allowed(&self, command: &Command) -> bool {
        (*self).next_state(command).is_some()
    }

    /// Returns the state resulting of the given command
    ///
    /// A `None` return value means the comand is not allowed.
    pub fn next_state(&self, command: &Command) -> Option<TransactionState> {
        match (*self, command) {
            (Unconnected, &Command::Connect) => Some(Connected),
            (Unconnected, _) => None,
            (DataSent, &Command::Message) => Some(HelloSent),
            (DataSent, _) => None,
            // Authentication
            (AuthenticateSent, &Command::AuthenticationResponse(_)) => Some(HelloSent),
            (AuthenticateSent, _) => None,
            (HelloSent, &Command::Authenticate(_, None)) => Some(AuthenticateSent),
            (HelloSent, &Command::Authenticate(_, Some(_))) => Some(HelloSent),
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
            // Email transaction
            (HelloSent, &Command::Mail(..)) => Some(MailSent),
            (MailSent, &Command::Recipient(..)) => Some(RecipientSent),
            (RecipientSent, &Command::Recipient(..)) => Some(RecipientSent),
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
    fn test_is_allowed() {
        assert!(!TransactionState::Unconnected.is_allowed(&Command::Noop));
        assert!(!TransactionState::DataSent.is_allowed(&Command::Noop));
        assert!(TransactionState::HelloSent.is_allowed(&Command::Mail("".to_string(), None)));
        assert!(!TransactionState::MailSent.is_allowed(&Command::Mail("".to_string(), None)));
    }

    #[test]
    fn test_next_state() {
        assert_eq!(TransactionState::MailSent.next_state(&Command::Noop), Some(TransactionState::MailSent));
        assert_eq!(TransactionState::HelloSent.next_state(&Command::Mail("".to_string(), None)),
                   Some(TransactionState::MailSent));
        assert_eq!(TransactionState::MailSent.next_state(&Command::Mail("".to_string(), None)), None);
    }
}
