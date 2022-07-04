////////////////////////////////////////////////////////////////////////////////
// Copyright (c) 2022 qianjunakasumi <i@qianjunakasumi.ren>                    /
//                     qianjunakasumi <qianjunakasumi@outlook.com>             /
//                     https://github.com/qianjunakasumi                       /
//                                                                             /
//     This Source Code Form is subject to the terms of the Mozilla Public     /
//     License, v. 2.0. If a copy of the MPL was not distributed with this     /
//     file, You can obtain one at http://mozilla.org/MPL/2.0/.                /
////////////////////////////////////////////////////////////////////////////////

package main

import (
	"strconv"
	"strings"
)

const SEP = `);
`
const HEAD = `// This file is automatically @generated by JceGenerator.
// It is not intended for manual editing.

use bytes::{Bytes, BytesMut};

use conch_jce::{JceReader, JceWriter};
use conch_jce::field::{JBool, JByte, JceStruct, JDouble, JFloat, JInt, JList, JLong, JMap, JShort, JSList, JString};`
const STRUCT = `
#[derive(Default)]
struct `
const STRUCTSTART = ` {
`
const STRUCTEND = `}

impl JceStruct<`
const IMPLSTART = `> for `
const IMPLMIDDLE1 = ` {
    fn s_to_bytes(&self, b: &mut BytesMut) {
        let mut w = JceWriter::new(`
const IMPMMIDDLE2 = `
    }

    fn s_from_bytes(&mut self, b: &mut Bytes) {
        let mut r = JceReader::with_tag(b, `
const END = `    }
}`

func format(j *JceSpec) (b strings.Builder) {
	t := strconv.FormatUint(uint64(j.Spec.StartTag), 10)
	b.WriteString(STRUCT)
	b.WriteString(j.Metadata.Name)
	b.WriteString(STRUCTSTART)
	b.WriteString(formatStruct(j))
	b.WriteString(STRUCTEND)
	b.WriteString(j.Metadata.Name)
	b.WriteString(IMPLSTART)
	b.WriteString(j.Metadata.Name)
	b.WriteString(IMPLMIDDLE1)
	b.WriteString(t)
	b.WriteString(SEP)
	b.WriteString(formatImplToBytes(j))
	b.WriteString(IMPMMIDDLE2)
	b.WriteString(t)
	b.WriteString(SEP)
	b.WriteString(formatImplFromBytes(j))
	b.WriteString(END)
	return
}

func formatStruct(j *JceSpec) string {
	var b strings.Builder
	for i, v := range j.Spec.Field {
		b.WriteString("    ")
		b.WriteString(v.Name)
		b.WriteString(": ")
		if !v.Option {
			b.WriteString(v.Type)
		} else {
			b.WriteString("Option<")
			b.WriteString(v.Type)
			b.WriteString(">")
		}

		b.WriteString(",")
		if i != len(j.Spec.Field) {
			b.WriteString(`
`)
		}
	}
	return b.String()
}

func formatImplToBytes(j *JceSpec) string {
	var b strings.Builder
	for i, v := range j.Spec.Field {
		if v.Tag != nil && *v.Tag != uint8(i)+j.Spec.StartTag {
			b.WriteString("        w.set_tag(")
			b.WriteString(strconv.FormatUint(uint64(*v.Tag), 10))
			b.WriteString(`);
`)
		}

		if !v.Option {
			b.WriteString("        w.put(&self.")
			b.WriteString(v.Name)
			b.WriteString(");")
		} else {
			b.WriteString("        match self.")
			b.WriteString(v.Name)
			b.WriteString(` {
            Some(v) => w.put(&v),
            None => {}
        }`)
		}
		b.WriteString(`
`)
	}
	b.WriteString("        w.flash(b);")

	return b.String()
}

func formatImplFromBytes(j *JceSpec) string {
	var b strings.Builder
	for i, v := range j.Spec.Field {
		if v.Tag != nil && *v.Tag != uint8(i)+j.Spec.StartTag {
			b.WriteString("        r.set_tag(")
			b.WriteString(strconv.FormatUint(uint64(*v.Tag), 10))
			b.WriteString(`);
`)
		}

		b.WriteString("        self.")
		b.WriteString(v.Name)
		b.WriteString(" = ")
		if !v.Option {
			b.WriteString("r.get();")
		} else {
			b.WriteString("r.get_optional();")
		}

		if i != len(j.Spec.Field) {
			b.WriteString(`
`)
		}
	}
	return b.String()
}
