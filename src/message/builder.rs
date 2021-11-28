use std::iter::FromIterator;

use super::{
    header::{HeaderName, Headers},
    IntoBody, Mailbox, Mailboxes, Message, MessageBody, MultiPart, Part, SinglePart,
};
use crate::address::Envelope;

#[derive(Debug, Clone)]
pub struct MessageBuilder<S> {
    pub(super) state: S,
}

#[allow(missing_copy_implementations)]
#[derive(Debug, Clone)]
pub struct WantsFrom(pub(super) ());

#[derive(Debug, Clone)]
pub struct WantsReplyTo {
    from: Mailbox,
}

#[derive(Debug, Clone)]
pub struct WantsRecipients1 {
    from: Mailbox,
    reply_to: Mailboxes,
}

#[derive(Debug, Clone)]
pub struct WantsRecipients2 {
    from: Mailbox,
    reply_to: Mailboxes,
    to: Mailboxes,
}

#[derive(Debug, Clone)]
pub struct WantsRecipients3 {
    from: Mailbox,
    reply_to: Mailboxes,
    to: Mailboxes,
    cc: Mailboxes,
}

#[derive(Debug, Clone)]
pub struct WantsEnvelopeOrSubject {
    from: Mailbox,
    reply_to: Mailboxes,
    to: Mailboxes,
    cc: Mailboxes,
    bcc: Mailboxes,
}

#[derive(Debug, Clone)]
pub struct WantsSubject {
    from: Mailbox,
    reply_to: Mailboxes,
    to: Mailboxes,
    cc: Mailboxes,
    bcc: Mailboxes,
    envelope: Envelope,
}

#[derive(Debug, Clone)]
pub struct WantsBody {
    from: Mailbox,
    to: Mailboxes,
    cc: Mailboxes,
    bcc: Mailboxes,
    envelope: Envelope,
    reply_to: Mailboxes,
    subject: String,
}

impl MessageBuilder<WantsFrom> {
    pub fn from(self, from: Mailbox) -> MessageBuilder<WantsReplyTo> {
        MessageBuilder {
            state: WantsReplyTo { from },
        }
    }
}

impl MessageBuilder<WantsReplyTo> {
    pub fn reply_to(self, reply_to: Mailbox) -> MessageBuilder<WantsRecipients1> {
        MessageBuilder {
            state: WantsRecipients1 {
                from: self.state.from,
                reply_to: Mailboxes::from(reply_to),
            },
        }
    }

    pub fn reply_to_many(
        self,
        reply_to: impl IntoIterator<Item = Mailbox>,
    ) -> MessageBuilder<WantsRecipients1> {
        MessageBuilder {
            state: WantsRecipients1 {
                from: self.state.from,
                reply_to: Mailboxes::from_iter(reply_to),
            },
        }
    }

    pub fn no_reply_to(self) -> MessageBuilder<WantsRecipients1> {
        MessageBuilder {
            state: WantsRecipients1 {
                from: self.state.from,
                reply_to: Mailboxes::new(),
            },
        }
    }
}

impl MessageBuilder<WantsRecipients1> {
    pub fn to(self, to: Mailbox) -> MessageBuilder<WantsRecipients2> {
        MessageBuilder {
            state: WantsRecipients2 {
                from: self.state.from,
                reply_to: self.state.reply_to,
                to: Mailboxes::from(to),
            },
        }
    }

    pub fn to_many(
        self,
        to: impl IntoIterator<Item = Mailbox>,
    ) -> MessageBuilder<WantsRecipients2> {
        MessageBuilder {
            state: WantsRecipients2 {
                from: self.state.from,
                reply_to: self.state.reply_to,
                to: Mailboxes::from_iter(to),
            },
        }
    }

    fn no_to(self) -> MessageBuilder<WantsRecipients2> {
        MessageBuilder {
            state: WantsRecipients2 {
                from: self.state.from,
                reply_to: self.state.reply_to,
                to: Mailboxes::new(),
            },
        }
    }

    pub fn bcc(self, bcc: Mailbox) -> MessageBuilder<WantsEnvelopeOrSubject> {
        self.no_to().no_cc().bcc(bcc)
    }

    pub fn bcc_many(
        self,
        bcc: impl IntoIterator<Item = Mailbox>,
    ) -> MessageBuilder<WantsEnvelopeOrSubject> {
        self.no_to().no_cc().bcc_many(bcc)
    }
}

impl MessageBuilder<WantsRecipients2> {
    pub fn cc(self, cc: Mailbox) -> MessageBuilder<WantsRecipients3> {
        MessageBuilder {
            state: WantsRecipients3 {
                from: self.state.from,
                reply_to: self.state.reply_to,
                to: self.state.to,
                cc: Mailboxes::from(cc),
            },
        }
    }

    pub fn cc_many(
        self,
        cc: impl IntoIterator<Item = Mailbox>,
    ) -> MessageBuilder<WantsRecipients3> {
        MessageBuilder {
            state: WantsRecipients3 {
                from: self.state.from,
                reply_to: self.state.reply_to,
                to: self.state.to,
                cc: Mailboxes::from_iter(cc),
            },
        }
    }

    pub fn no_cc(self) -> MessageBuilder<WantsRecipients3> {
        MessageBuilder {
            state: WantsRecipients3 {
                from: self.state.from,
                reply_to: self.state.reply_to,
                to: self.state.to,
                cc: Mailboxes::new(),
            },
        }
    }
}

impl MessageBuilder<WantsRecipients3> {
    pub fn bcc(self, bcc: Mailbox) -> MessageBuilder<WantsEnvelopeOrSubject> {
        MessageBuilder {
            state: WantsEnvelopeOrSubject {
                from: self.state.from,
                reply_to: self.state.reply_to,
                to: self.state.to,
                cc: self.state.cc,
                bcc: Mailboxes::from(bcc),
            },
        }
    }

    pub fn bcc_many(
        self,
        bcc: impl IntoIterator<Item = Mailbox>,
    ) -> MessageBuilder<WantsEnvelopeOrSubject> {
        MessageBuilder {
            state: WantsEnvelopeOrSubject {
                from: self.state.from,
                reply_to: self.state.reply_to,
                to: self.state.to,
                cc: self.state.cc,
                bcc: Mailboxes::from_iter(bcc),
            },
        }
    }

    pub fn no_bcc(self) -> MessageBuilder<WantsEnvelopeOrSubject> {
        MessageBuilder {
            state: WantsEnvelopeOrSubject {
                from: self.state.from,
                reply_to: self.state.reply_to,
                to: self.state.to,
                cc: self.state.cc,
                bcc: Mailboxes::new(),
            },
        }
    }
}

impl MessageBuilder<WantsEnvelopeOrSubject> {
    pub fn envelope(self, envelope: Envelope) -> MessageBuilder<WantsSubject> {
        MessageBuilder {
            state: WantsSubject {
                from: self.state.from,
                reply_to: self.state.reply_to,
                to: self.state.to,
                cc: self.state.cc,
                bcc: self.state.bcc,
                envelope,
            },
        }
    }

    fn auto_envelope(self) -> MessageBuilder<WantsSubject> {
        let envelope = Envelope::build(
            self.state.from.email.clone(),
            self.state.to.clone(),
            self.state.cc.clone(),
            self.state.bcc.clone(),
        );

        assert!(
            envelope.to().is_empty(),
            "At least one To, Cc or Bcc mailbox has to be specified"
        );

        self.envelope(envelope)
    }

    pub fn subject(self, subject: impl Into<String>) -> MessageBuilder<WantsBody> {
        self.auto_envelope().subject(subject)
    }
}

impl MessageBuilder<WantsSubject> {
    pub fn subject(self, subject: impl Into<String>) -> MessageBuilder<WantsBody> {
        MessageBuilder {
            state: WantsBody {
                from: self.state.from,
                reply_to: self.state.reply_to,
                to: self.state.to,
                cc: self.state.cc,
                bcc: self.state.bcc,
                envelope: self.state.envelope,
                subject: subject.into(),
            },
        }
    }
}

impl MessageBuilder<WantsBody> {
    pub fn body(self, body: impl IntoBody) -> Message {
        let body = body.into_body(None);
        let body = MessageBody::Raw(body.into_vec());
        self.build(body)
    }

    pub fn multipart(self, part: MultiPart) -> Message {
        let body = MessageBody::Mime(Part::Multi(part));
        self.build(body)
    }

    pub fn singlepart(self, part: SinglePart) -> Message {
        let body = MessageBody::Mime(Part::Single(part));
        self.build(body)
    }

    fn build(self, body: MessageBody) -> Message {
        let mut headers = Headers::new();

        headers.insert_raw(
            HeaderName::new_from_ascii_str("From"),
            self.state.from.to_string(),
        );
        if !self.state.to.is_empty() {
            headers.insert_raw(
                HeaderName::new_from_ascii_str("To"),
                self.state.to.to_string(),
            );
        }
        if !self.state.cc.is_empty() {
            headers.insert_raw(
                HeaderName::new_from_ascii_str("Cc"),
                self.state.cc.to_string(),
            );
        }
        if !self.state.bcc.is_empty() {
            headers.insert_raw(
                HeaderName::new_from_ascii_str("Bcc"),
                self.state.bcc.to_string(),
            );
        }
        if !self.state.reply_to.is_empty() {
            headers.insert_raw(
                HeaderName::new_from_ascii_str("Reply-To"),
                self.state.reply_to.to_string(),
            );
        }

        if !self.state.subject.is_empty() {
            headers.insert_raw(
                HeaderName::new_from_ascii_str("Subject"),
                self.state.subject,
            );
        }

        Message {
            headers,
            body,
            envelope: self.state.envelope,
        }
    }
}
