//! BSON document decoder.
//!
//! Upstream reference: `json-pack/src/bson/BsonDecoder.ts`
//!
//! BSON is a little-endian binary format.

use super::values::{
    BsonBinary, BsonDbPointer, BsonDecimal128, BsonJavascriptCode, BsonJavascriptCodeWithScope,
    BsonObjectId, BsonSymbol, BsonTimestamp, BsonValue,
};

/// BSON document decoder.
pub struct BsonDecoder {
    data: Vec<u8>,
    x: usize,
}

impl Default for BsonDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl BsonDecoder {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            x: 0,
        }
    }

    /// Decodes a BSON document from bytes.
    pub fn decode(&mut self, data: &[u8]) -> Vec<(String, BsonValue)> {
        self.data = data.to_vec();
        self.x = 0;
        self.read_document()
    }

    fn assert_size(&self, n: usize) {
        if self.x + n > self.data.len() {
            panic!("BSON decode out of bounds");
        }
    }

    fn u8(&mut self) -> u8 {
        self.assert_size(1);
        let val = self.data[self.x];
        self.x += 1;
        val
    }

    fn i32_le(&mut self) -> i32 {
        self.assert_size(4);
        let val = i32::from_le_bytes([
            self.data[self.x],
            self.data[self.x + 1],
            self.data[self.x + 2],
            self.data[self.x + 3],
        ]);
        self.x += 4;
        val
    }

    fn i64_le(&mut self) -> i64 {
        self.assert_size(8);
        let val = i64::from_le_bytes([
            self.data[self.x],
            self.data[self.x + 1],
            self.data[self.x + 2],
            self.data[self.x + 3],
            self.data[self.x + 4],
            self.data[self.x + 5],
            self.data[self.x + 6],
            self.data[self.x + 7],
        ]);
        self.x += 8;
        val
    }

    fn f64_le(&mut self) -> f64 {
        self.assert_size(8);
        let val = f64::from_le_bytes([
            self.data[self.x],
            self.data[self.x + 1],
            self.data[self.x + 2],
            self.data[self.x + 3],
            self.data[self.x + 4],
            self.data[self.x + 5],
            self.data[self.x + 6],
            self.data[self.x + 7],
        ]);
        self.x += 8;
        val
    }

    fn buf(&mut self, n: usize) -> Vec<u8> {
        self.assert_size(n);
        let data = self.data[self.x..self.x + n].to_vec();
        self.x += n;
        data
    }

    fn utf8(&mut self, n: usize) -> String {
        let bytes = self.buf(n);
        String::from_utf8(bytes).unwrap_or_default()
    }

    fn read_document(&mut self) -> Vec<(String, BsonValue)> {
        let document_size = self.i32_le() as usize;
        let start_pos = self.x; // position after the 4-byte size field
        let end_pos = start_pos + document_size - 4 - 1; // before terminating null
        let mut fields: Vec<(String, BsonValue)> = Vec::new();

        while self.x < end_pos {
            let element_type = self.u8();
            if element_type == 0 {
                break;
            }
            let key = self.read_cstring();
            let value = self.read_element_value(element_type);
            fields.push((key, value));
        }

        // Skip to end of document (including terminating null)
        if self.x <= end_pos {
            self.x = start_pos + document_size - 4;
        }

        fields
    }

    fn read_cstring(&mut self) -> String {
        let start = self.x;
        while self.x < self.data.len() && self.data[self.x] != 0 {
            self.x += 1;
        }
        let s = String::from_utf8(self.data[start..self.x].to_vec()).unwrap_or_default();
        self.x += 1; // skip null terminator
        s
    }

    fn read_string(&mut self) -> String {
        let length = self.i32_le() as usize;
        if length == 0 {
            return String::new();
        }
        let s = self.utf8(length - 1); // -1: length includes null terminator
        self.x += 1; // skip null terminator
        s
    }

    fn read_element_value(&mut self, typ: u8) -> BsonValue {
        match typ {
            0x01 => BsonValue::Float(self.f64_le()),
            0x02 => BsonValue::Str(self.read_string()),
            0x03 => BsonValue::Document(self.read_document()),
            0x04 => BsonValue::Array(self.read_array()),
            0x05 => self.read_binary(),
            0x06 => BsonValue::Undefined,
            0x07 => BsonValue::ObjectId(self.read_object_id()),
            0x08 => BsonValue::Boolean(self.u8() == 1),
            0x09 => BsonValue::DateTime(self.i64_le()),
            0x0a => BsonValue::Null,
            0x0b => self.read_regex(),
            0x0c => self.read_db_pointer(),
            0x0d => BsonValue::JavaScriptCode(BsonJavascriptCode {
                code: self.read_string(),
            }),
            0x0e => BsonValue::Symbol(BsonSymbol {
                symbol: self.read_string(),
            }),
            0x0f => self.read_code_with_scope(),
            0x10 => BsonValue::Int32(self.i32_le()),
            0x11 => self.read_timestamp(),
            0x12 => BsonValue::Int64(self.i64_le()),
            0x13 => BsonValue::Decimal128(BsonDecimal128 { data: self.buf(16) }),
            0xff => BsonValue::MinKey,
            0x7f => BsonValue::MaxKey,
            _ => panic!("Unsupported BSON type: 0x{:02x}", typ),
        }
    }

    fn read_array(&mut self) -> Vec<BsonValue> {
        let fields = self.read_document();
        // Sort by numeric key and extract values
        let mut indexed: Vec<(usize, BsonValue)> = fields
            .into_iter()
            .map(|(k, v)| (k.parse::<usize>().unwrap_or(0), v))
            .collect();
        indexed.sort_by_key(|(i, _)| *i);
        indexed.into_iter().map(|(_, v)| v).collect()
    }

    fn read_binary(&mut self) -> BsonValue {
        let length = self.i32_le() as usize;
        let subtype = self.u8();
        let data = self.buf(length);
        BsonValue::Binary(BsonBinary { subtype, data })
    }

    fn read_object_id(&mut self) -> BsonObjectId {
        let bytes = self.buf(12);
        // Timestamp: 4 bytes big-endian
        let timestamp = ((bytes[0] as u32) << 24)
            | ((bytes[1] as u32) << 16)
            | ((bytes[2] as u32) << 8)
            | (bytes[3] as u32);
        // Process: 5 bytes (4 LE + 1 high)
        let lo32 = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as u64;
        let hi8 = bytes[8] as u64;
        let process = lo32 | (hi8 << 32);
        // Counter: 3 bytes big-endian
        let counter = ((bytes[9] as u32) << 16) | ((bytes[10] as u32) << 8) | (bytes[11] as u32);
        BsonObjectId {
            timestamp,
            process,
            counter,
        }
    }

    fn read_regex(&mut self) -> BsonValue {
        let pattern = self.read_cstring();
        let flags = self.read_cstring();
        BsonValue::Regex(pattern, flags)
    }

    fn read_db_pointer(&mut self) -> BsonValue {
        let name = self.read_string();
        let id = self.read_object_id();
        BsonValue::DbPointer(BsonDbPointer { name, id })
    }

    fn read_code_with_scope(&mut self) -> BsonValue {
        let _total_len = self.i32_le(); // skip total length
        let code = self.read_string();
        let scope = self.read_document();
        BsonValue::JavaScriptCodeWithScope(BsonJavascriptCodeWithScope { code, scope })
    }

    fn read_timestamp(&mut self) -> BsonValue {
        let increment = self.i32_le();
        let timestamp = self.i32_le();
        BsonValue::Timestamp(BsonTimestamp {
            increment,
            timestamp,
        })
    }
}
