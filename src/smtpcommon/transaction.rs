// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! SMTP commands and ESMTP features library

use std::fmt;
use std::fmt::{Show, Formatter};
use smtpcommon::command;
use smtpcommon::command::SmtpCommand;

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
    DataSent
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
                DataSent => "DataSent"
            }.as_bytes()
        )
    }
}

impl TransactionState {
    /// bla bla
    pub fn is_command_possible(&self, command: SmtpCommand) -> bool {
        match (*self, command) {
            (Unconnected, command::Connect) => true,
            (Unconnected, _) => false,
            // Only the message content can be sent in this state
            (DataSent, _) => false,
            // Commands that can be issued everytime
            (_, command::ExtendedHello(_)) => true,
            (_, command::Hello(_)) => true,
            (_, command::Reset) => true,
            (_, command::Verify(_, _)) => true,
            (_, command::Expand(_, _)) => true,
            (_, command::Help(_)) => true,
            (_, command::Noop) => true,
            (_, command::Quit) => true,
            // Commands that require a particular state
            (HelloSent, command::Mail(_, _)) => true,
            (MailSent, command::Recipient(_, _)) => true,
            (RecipientSent, command::Recipient(_, _)) => true,
            (RecipientSent, command::Data) => true,
            // Everything else
            (_, _) => false
        }
    }

    /// a method
    pub fn next_state(&mut self, command: SmtpCommand) -> Option<TransactionState> {
        match (*self, command) {
            (Unconnected, command::Connect) => Some(Connected),
            (Unconnected, _) => None,
            (DataSent, _) => None,
            // Commands that can be issued everytime
            (_, command::ExtendedHello(_)) => Some(HelloSent),
            (_, command::Hello(_)) => Some(HelloSent),
            (Connected, command::Reset) => Some(Connected),
            (_, command::Reset) => Some(HelloSent),
            (state, command::Verify(_, _)) => Some(state),
            (state, command::Expand(_, _)) => Some(state),
            (state, command::Help(_)) => Some(state),
            (state, command::Noop) => Some(state),
            (_, command::Quit) => Some(Unconnected),
            // Commands that require a particular state
            (HelloSent, command::Mail(_, _)) => Some(MailSent),
            (MailSent, command::Recipient(_, _)) => Some(RecipientSent),
            (RecipientSent, command::Recipient(_, _)) => Some(RecipientSent),
            (RecipientSent, command::Data) => Some(DataSent),
            // Everything else
            (_, _) => None
        }
    }
}

#[cfg(test)]
mod test {
    use smtpcommon::command;

    #[test]
    fn test_transaction_state_is_command_possible() {
        assert!(!super::Unconnected.is_command_possible(command::Noop));
        assert!(!super::DataSent.is_command_possible(command::Noop));
        assert!(super::HelloSent.is_command_possible(command::Mail("".to_string(), None)));
        assert!(!super::MailSent.is_command_possible(command::Mail("".to_string(), None)));
    }

    #[test]
    fn test_super_next_state() {
        assert_eq!(super::MailSent.next_state(command::Noop), Some(super::MailSent));
        assert_eq!(super::HelloSent.next_state(command::Mail("".to_string(), None)), Some(super::MailSent));
        assert_eq!(super::MailSent.next_state(command::Mail("".to_string(), None)), None);
    }
}
