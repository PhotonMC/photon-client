//! `DebugQueryHandler`: the questions F3+I asks the server about a block entity or an entity, and
//! the answers it gets back.

use super::*;
use photon_core::ids::EntityId;
use photon_core::nbt::Field;

/// What a query was asked about, kept for the answer.
#[derive(Debug, Clone, PartialEq)]
pub enum DebugQuery {
    /// A block entity: where it is and the serialized state of its block.
    Block { pos: (i32, i32, i32), state: String },
    /// An entity: its id, its type's name and where it stood.
    Entity {
        id: EntityId,
        name: String,
        position: [f64; 3],
    },
}

/// The tag the server answered a query with.
#[derive(Debug, Clone, PartialEq)]
pub struct DebugQueryAnswer {
    pub query: DebugQuery,
    pub tag: Option<Vec<Field>>,
}

/// `DebugQueryHandler.transactionId` and its callback.
#[derive(Debug)]
pub(super) struct DebugQueries {
    transaction: i32,
    pending: Option<DebugQuery>,
    answers: Vec<DebugQueryAnswer>,
}

impl Default for DebugQueries {
    fn default() -> Self {
        Self {
            transaction: -1,
            pending: None,
            answers: Vec::new(),
        }
    }
}

impl DebugQueries {
    /// `startTransaction`: the new query replaces the one waiting, and gets the next number.
    fn start(&mut self, query: DebugQuery) -> i32 {
        self.pending = Some(query);
        self.transaction = self.transaction.wrapping_add(1);
        self.transaction
    }

    /// `handleResponse`: whether the answer is for the query that waits.
    pub(super) fn answer(&mut self, transaction: i32, tag: Option<Vec<Field>>) -> bool {
        if self.transaction != transaction {
            return false;
        }
        let Some(query) = self.pending.take() else {
            return false;
        };
        self.answers.push(DebugQueryAnswer { query, tag });
        true
    }
}

impl PlaySession {
    /// `queryBlockEntityTag`.
    pub fn query_block_entity_tag(
        &mut self,
        pos: (i32, i32, i32),
        state: String,
    ) -> Result<(), ConnectionError> {
        let transaction = self
            .social
            .debug_queries
            .start(DebugQuery::Block { pos, state });
        self.send(
            PLAY_SERVERBOUND_BLOCK_ENTITY_TAG_QUERY_PACKET_ID,
            |output| {
                wire::write_varint(transaction, output);
                output.extend_from_slice(&pack_block_position(pos).to_be_bytes());
                Ok(())
            },
        )
    }

    /// `queryEntityTag`.
    pub fn query_entity_tag(
        &mut self,
        id: EntityId,
        name: String,
        position: [f64; 3],
    ) -> Result<(), ConnectionError> {
        let transaction =
            self.social
                .debug_queries
                .start(DebugQuery::Entity { id, name, position });
        self.send(PLAY_SERVERBOUND_ENTITY_TAG_QUERY_PACKET_ID, |output| {
            wire::write_varint(transaction, output);
            wire::write_varint(id.0, output);
            Ok(())
        })
    }

    /// The answers that came since the last call.
    pub fn take_query_answers(&mut self) -> Vec<DebugQueryAnswer> {
        std::mem::take(&mut self.social.debug_queries.answers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::play::profile_tests::session;
    use crate::connection::transport::Transport;

    fn sent(server: &std::net::TcpListener) -> (i32, Vec<u8>) {
        let (stream, _) = server.accept().expect("accept");
        let mut stream = Transport::from(stream);
        let packet = crate::connection::read_packet(&mut stream, None).expect("packet");
        let (id, body) = split_packet(&packet).expect("split");
        (id, body.to_vec())
    }

    #[test]
    fn the_answer_to_the_last_query_is_kept_and_an_older_one_is_not() {
        let (mut session, server) = session();
        session
            .query_block_entity_tag((1, 64, -2), "minecraft:chest".to_owned())
            .expect("sent");
        let (id, body) = sent(&server);
        assert_eq!(id, PLAY_SERVERBOUND_BLOCK_ENTITY_TAG_QUERY_PACKET_ID);
        let mut expected = vec![0];
        expected.extend_from_slice(&pack_block_position((1, 64, -2)).to_be_bytes());
        assert_eq!(body, expected);
        session
            .query_entity_tag(EntityId(300), "minecraft:pig".to_owned(), [1.0, 2.0, 3.0])
            .expect("sent");
        // Transaction 1 now; the first query was dropped.
        let queries = &mut session.social.debug_queries;
        assert!(!queries.answer(0, None));
        assert!(queries.answer(1, Some(Vec::new())));
        assert!(!queries.answer(1, None));
        let answers = session.take_query_answers();
        assert_eq!(answers.len(), 1);
        assert!(matches!(&answers[0].query, DebugQuery::Entity { id, .. } if id.0 == 300));
        assert_eq!(answers[0].tag, Some(Vec::new()));
    }
}
