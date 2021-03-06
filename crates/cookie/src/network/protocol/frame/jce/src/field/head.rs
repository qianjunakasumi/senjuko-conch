////////////////////////////////////////////////////////////////////////////////
// Copyright (c) 2022 qianjunakasumi <i@qianjunakasumi.ren>                    /
//                     qianjunakasumi <qianjunakasumi@outlook.com>             /
//                     https://github.com/qianjunakasumi                       /
//                                                                             /
//     This Source Code Form is subject to the terms of the Mozilla Public     /
//     License, v. 2.0. If a copy of the MPL was not distributed with this     /
//     file, You can obtain one at http://mozilla.org/MPL/2.0/.                /
////////////////////////////////////////////////////////////////////////////////

use bytes::{Buf, BufMut, Bytes, BytesMut};

use super::{BYTE, DOUBLE, FLOAT, INT, JceType, JInt, LIST, LONG, MAP, SHORT, SIMPLE_LIST, STRING1, STRING4, STRUCT_BEGIN, STRUCT_END, TYPE_ERR};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct HeadData {
    pub r#type: u8,
    pub tag: u8,
}

impl HeadData {
    pub fn new(r#type: u8, tag: u8) -> HeadData { HeadData { r#type, tag } }

    pub fn parse(b: &mut Bytes) -> HeadData {
        let f = b.get_u8();
        let t = (f & 240) >> 4;
        HeadData { r#type: f & 15, tag: if t != 15 { t } else { b.get_u8() } }
    }

    pub fn format(&self, b: &mut BytesMut, additional: usize) {
        b.reserve(2 + additional);
        if self.tag <= 14 {
            b.put_u8(self.r#type | (self.tag << 4));
        } else {
            b.put_u8(self.r#type | 240);
            b.put_u8(self.tag);
        }
    }
}

impl HeadData {
    pub fn parse_ttl4(b: &mut Bytes) -> usize {
        let head = HeadData::parse(b);
        if head.tag != 0 { panic!("{}", TYPE_ERR) }
        JInt::from_bytes(b, head.r#type) as usize
    }

    pub fn skip_value(&self, b: &mut Bytes) {
        if self.r#type > 13 {
            panic!("{}", TYPE_ERR);
        }

        let len = match self.r#type {
            BYTE => 1,
            SHORT => 2,
            INT => 4,
            LONG => 8,
            FLOAT => 4,
            DOUBLE => 8,
            STRING1 => b.get_u8() as usize,
            STRING4 => b.get_i32() as usize,
            MAP => {
                let len = HeadData::parse_ttl4(b);
                let mut i = 0;
                while i < len {
                    HeadData::parse(b).skip_value(b); // K
                    HeadData::parse(b).skip_value(b); // V
                    i += 1;
                }
                0
            }
            LIST => {
                let len = HeadData::parse_ttl4(b);
                let mut i = 0;
                while i < len {
                    HeadData::parse(b).skip_value(b);
                    i += 1;
                }
                0
            }
            STRUCT_BEGIN => {
                let mut h = HeadData::parse(b);
                while h.r#type != STRUCT_END {
                    h.skip_value(b);
                    h = HeadData::parse(b);
                }
                0
            }
            SIMPLE_LIST => 1 + HeadData::parse_ttl4(b), // 1: 0 type 0 tag head
            _ => 0, // STRUCT_END + ZERO_TAG
        };
        b.advance(len);
    }
}

#[cfg(test)]
mod tests {
    use bytes::{Bytes, BytesMut};

    use super::HeadData;

    const A: HeadData = HeadData { r#type: 0, tag: 0 };
    const B: HeadData = HeadData { r#type: 1, tag: 0 };
    const C: HeadData = HeadData { r#type: 1, tag: 2 };
    const D: HeadData = HeadData { r#type: 2, tag: 8 };
    const E: HeadData = HeadData { r#type: 4, tag: 24 };

    #[test]
    fn parse() {
        assert_eq!(HeadData::parse(&mut Bytes::from(vec![0])), A);
        assert_eq!(HeadData::parse(&mut Bytes::from(vec![1])), B);
        assert_eq!(HeadData::parse(&mut Bytes::from(vec![33])), C);
        assert_eq!(HeadData::parse(&mut Bytes::from(vec![130])), D);
        assert_eq!(HeadData::parse(&mut Bytes::from(vec![244, 24])), E);
    }

    #[test]
    fn format() {
        let mut b = BytesMut::new();
        A.format(&mut b, 0);
        assert_eq!(b.to_vec(), vec![0]);

        let mut b = BytesMut::new();
        B.format(&mut b, 0);
        assert_eq!(b.to_vec(), vec![1]);

        let mut b = BytesMut::new();
        C.format(&mut b, 0);
        assert_eq!(b.to_vec(), vec![33]);

        let mut b = BytesMut::new();
        D.format(&mut b, 0);
        assert_eq!(b.to_vec(), vec![130]);

        let mut b = BytesMut::new();
        E.format(&mut b, 0);
        assert_eq!(b.to_vec(), vec![244, 24]);
    }
}
