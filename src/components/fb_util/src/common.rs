// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use std::io::Read;

use bytes::{Buf, BufMut};
use flatbuffers;
use tonic::{
    codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder},
    Status,
};

pub struct FlatBufferBytes(Vec<u8>);

impl FlatBufferBytes {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    pub fn serialize<'buf, T: flatbuffers::Follow<'buf> + 'buf>(
        mut builder: flatbuffers::FlatBufferBuilder<'buf>,
        root_offset: flatbuffers::WIPOffset<T>,
    ) -> Self {
        builder.finish(root_offset, None);
        let (mut data, head) = builder.collapse();
        Self(data.drain(head..).collect())
    }

    pub fn deserialize<'buf, T: flatbuffers::Follow<'buf> + flatbuffers::Verifiable + 'buf>(
        &'buf self,
    ) -> Result<T::Inner, Box<dyn std::error::Error>> {
        flatbuffers::root::<T>(self.0.as_slice())
            .map_err(|x| Box::new(x) as Box<dyn std::error::Error>)
    }
}

#[derive(Debug)]
pub struct FlatBufferEncoder();

impl Encoder for FlatBufferEncoder {
    type Error = Status;
    type Item = FlatBufferBytes;

    fn encode(&mut self, item: Self::Item, buf: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        buf.put_slice(item.0.as_slice());
        Ok(())
    }
}

#[derive(Debug)]
pub struct FlatBufferDecoder();

impl Decoder for FlatBufferDecoder {
    type Error = Status;
    type Item = FlatBufferBytes;

    fn decode(&mut self, buf: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        if !buf.has_remaining() {
            return Ok(None);
        }
        let mut data: Vec<u8> = Vec::new();
        buf.reader()
            .read_to_end(&mut data)
            .map_err(|e| Status::internal(e.to_string()))?;
        let item = FlatBufferBytes::new(data);
        Ok(Some(item))
    }
}

/// A [`Codec`] that implements `application/grpc+json` via the serde library.
#[derive(Debug, Clone, Default)]
pub struct FlatBufferCodec();

impl Codec for FlatBufferCodec {
    type Decode = FlatBufferBytes;
    type Decoder = FlatBufferDecoder;
    type Encode = FlatBufferBytes;
    type Encoder = FlatBufferEncoder;

    fn encoder(&mut self) -> Self::Encoder {
        FlatBufferEncoder()
    }

    fn decoder(&mut self) -> Self::Decoder {
        FlatBufferDecoder()
    }
}
