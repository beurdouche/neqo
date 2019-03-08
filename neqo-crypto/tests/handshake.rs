#![allow(dead_code)]

use neqo_crypto::*;
use std::mem;

pub const NOW: u64 = 20;

pub fn forward_records(agent: &mut SecretAgent, records_in: RecordList) -> Res<RecordList> {
    let mut expected_state = match agent.state() {
        HandshakeState::New => HandshakeState::New,
        _ => HandshakeState::InProgress,
    };
    let mut records_out: RecordList = Default::default();
    for record in records_in.into_iter() {
        assert_eq!(records_out.len(), 0);
        assert_eq!(*agent.state(), expected_state);

        let (_state, rec_out) = agent.handshake_raw(NOW, Some(record))?;
        records_out = rec_out;
        expected_state = HandshakeState::InProgress;
    }
    Ok(records_out)
}

fn handshake(client: &mut SecretAgent, server: &mut SecretAgent) {
    let mut a = client;
    let mut b = server;
    let (_, mut records) = a.handshake_raw(NOW, None).unwrap();
    let is_done = |agent: &mut SecretAgent| match *agent.state() {
        HandshakeState::Complete | HandshakeState::Failed(_) => true,
        _ => false,
    };
    while !is_done(a) || !is_done(b) {
        records = match forward_records(&mut b, records) {
            Ok(r) => r,
            _ => {
                // TODO(mt) take the alert generated by the failed handshake
                // and allow it to be sent to the peer.
                return;
            }
        };

        if *b.state() == HandshakeState::AuthenticationPending {
            b.authenticated();
            let (_, rec) = b.handshake_raw(NOW, None).unwrap();
            records = rec;
        }
        b = mem::replace(&mut a, b);
    }
}

pub fn connect(client: &mut SecretAgent, server: &mut SecretAgent) {
    handshake(client, server);
    assert_eq!(*client.state(), HandshakeState::Complete);
    assert_eq!(*server.state(), HandshakeState::Complete);
}

pub fn connect_fail(client: &mut SecretAgent, server: &mut SecretAgent) {
    handshake(client, server);
    assert_ne!(*client.state(), HandshakeState::Complete);
    assert_ne!(*server.state(), HandshakeState::Complete);
}
