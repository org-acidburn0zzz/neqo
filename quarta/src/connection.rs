use std::collections::{HashMap, HashSet};

use crate::data::Data;
use crate::frame::{decode_frame, Frame};
use crate::stream::Stream;
use crate::{Error, Res};

#[allow(unused_variables)]
#[derive(Debug, Default)]
pub struct Connection {
    max_data: Option<u64>,
    max_streams: Option<u64>,
    highest_stream: Option<u64>,
    connection_ids: HashSet<(u64, Vec<u8>)>, // (sequence number, connection id)
    max_used_stream_id: Option<u64>,
    streams: HashMap<u64, Stream>,      // stream id, stream
    outgoing_packets: Vec<(u64, Data)>, // (offset, data)
}

impl Connection {
    pub fn new() -> Connection {
        Connection::default()
    }

    pub fn process_inbound_datagram(&mut self, frame: &[u8]) -> Res<()> {
        let mut data = Data::from_slice(frame);
        let frame = decode_frame(&mut data)?;

        #[allow(unused_variables)]
        match frame {
            Frame::Padding => {
                println!("padding!");
            }
            Frame::Ping => {} // TODO generate ack
            Frame::Ack {
                largest_acknowledged,
                ack_delay,
                first_ack_range,
                ack_ranges,
            } => {} // TODO remove acked ranges from list of in-flight packets
            Frame::ResetStream {
                stream_id,
                application_error_code,
                final_size,
            } => {} // TODO reset a stream
            Frame::StopSending {
                application_error_code,
            } => {} // TODO stop sending on a stream
            Frame::Crypto { offset, data } => {} // TODO pass to crypto handling code
            Frame::NewToken { token } => {} // TODO stick the new token somewhere
            Frame::Stream {
                fin,
                stream_id,
                offset,
                data,
            } => {
                self.process_inbound_stream_frame(fin, stream_id, offset, data);
            }
            Frame::MaxData { maximum_data } => {} // TODO set self.max_data?
            Frame::MaxStreamData {
                stream_id,
                maximum_stream_data,
            } => {} // TODO lookup stream and modify its max_stream_data
            Frame::MaxStreams {
                stream_type,
                maximum_streams,
            } => {} // TODO adjust self.max_streams?
            Frame::DataBlocked { data_limit } => {} // TODO use as input to flow control algorithms
            Frame::StreamDataBlocked {
                stream_id,
                stream_data_limit,
            } => {} // TODO do something
            Frame::StreamsBlocked {
                stream_type,
                stream_limit,
            } => {} // TODO do something
            Frame::NewConnectionId {
                sequence_number,
                connection_id,
                stateless_reset_token,
            } => {
                self.connection_ids.insert((sequence_number, connection_id));
            }
            Frame::RetireConnectionId { sequence_number } => {} // TODO remove from list of connection IDs
            Frame::PathChallenge { data } => {}                 // TODO generate PATH_RESPONSE
            Frame::PathResponse { data } => {}                  // TODO do something
            Frame::ConnectionClose {
                close_type,
                error_code,
                frame_type,
                reason_phrase,
            } => {} // TODO close the connection
        };

        Ok(())
    }

    pub fn process_inbound_stream_frame(
        &mut self,
        fin: bool,
        stream_id: u64,
        offset: u64,
        data: Vec<u8>,
    ) {
        // TODO check against list of ooo frames and maybe make some data available
    }

    // Returns new stream id
    pub fn stream_create(&mut self) -> u64 {
        let next_stream_id = self.max_used_stream_id.unwrap_or_default() + 1;
        self.streams.insert(next_stream_id, Stream::new());
        self.max_used_stream_id = Some(next_stream_id);
        next_stream_id
    }

    pub fn stream_write(&mut self, stream_id: u64, data: Data) -> Res<()> {
        let stream = self
            .streams
            .get_mut(&stream_id)
            .ok_or_else(|| return Error::ErrInvalidStreamId)?;

        let remaining = data.remaining() as u64;
        self.outgoing_packets.push((stream.offset(), data)); // TODO give needed info to make a STREAM frame
        stream.add_to_offset(remaining);

        // TODO poke packet scheduler to maybe send some outgoing packets

        Ok(())
    }
}
