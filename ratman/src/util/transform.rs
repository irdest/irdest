// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use libratman::types::{
    api::{Send, Send_Type},
    Address, Id, Message, ApiRecipient, TimePair,
};

/// Turn an API `Send` to a `Message`
pub(crate) fn send_to_message(s: Send) -> Vec<Message> {
    // Take the set of recipients from the message and turn it into a
    // set of Ratman recipients
    let recipient: Vec<_> = match s.field_type {
        Send_Type::DEFAULT => s
            .get_msg()
            .get_recipient()
            .get_std()
            .get_standard()
            .into_iter()
            .map(|addr| ApiRecipient::Standard(vec![Address::from_bytes(&addr)]))
            .collect(),
        Send_Type::FLOOD => vec![ApiRecipient::Flood(Address::from_bytes(s.scope.as_slice()))],
    };
    let timesig = TimePair::sending();

    // Then create a new message for each recipient (if the type is
    // "flood" then only a single message gets created)
    recipient
        .into_iter()
        .map(|recipient| Message {
            id: Id::random(),
            sender: Address::from_bytes(s.get_msg().sender.as_slice()),
            recipient,
            payload: s.get_msg().payload.clone(),
            time: timesig.clone(),
            signature: s.get_msg().signature.clone(),
        })
        .collect()
}
