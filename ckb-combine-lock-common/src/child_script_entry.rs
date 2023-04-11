extern crate alloc;

use crate::{combine_lock_mol::ChildScript, error::Error};
use alloc::{string::String, vec::Vec};
use ckb_std::ckb_types::core::ScriptHashType;
use ckb_std::ckb_types::prelude::*;
use core::convert::From;
use core::result::Result;
use hex::{decode, encode_upper};
use molecule::bytes::Bytes;

pub struct ChildScriptEntry {
    pub code_hash: [u8; 32],
    pub hash_type: ScriptHashType,
    pub witness_index: u16,
    pub script_args: Vec<u8>,
}

impl From<ChildScript> for ChildScriptEntry {
    fn from(value: ChildScript) -> Self {
        let code_hash = value.code_hash().unpack();
        let hash_type_u8: u8 = value.hash_type().into();
        let args: Bytes = value.args().unpack();
        let hash_type = {
            match hash_type_u8 {
                0 => ScriptHashType::Data,
                1 => ScriptHashType::Type,
                2 => ScriptHashType::Data1,
                _ => panic!("wrong hash_type"),
            }
        };
        Self {
            code_hash,
            hash_type: hash_type,
            witness_index: 0,
            script_args: args.into(),
        }
    }
}

impl ChildScriptEntry {
    pub fn from_str(args: &str) -> Result<Self, Error> {
        // check string
        for c in args.as_bytes() {
            if !Self::check_char(c.clone() as char) {
                return Err(Error::WrongHex);
            }
        }

        let datas: Vec<&str> = args.split(':').map(|f| f).collect();
        if datas.len() != 4 {
            return Err(Error::WrongHex);
        }

        if datas[1].len() != 2 {
            return Err(Error::WrongHex);
        }

        if datas[2].len() != 4 {
            return Err(Error::WrongHex);
        }

        let code_hash: [u8; 32] = {
            let vec = decode(datas[0]).map_err(|_| Error::WrongHex)?;
            vec.try_into().map_err(|_| Error::WrongHex)?
        };

        let hash_type = {
            let slice = decode(datas[1]).map_err(|_| Error::WrongHex)?;
            let array: [u8; 1] = slice.try_into().map_err(|_| Error::WrongHex)?;
            let value = u8::from_le_bytes(array);
            match value {
                0 => ScriptHashType::Data,
                1 => ScriptHashType::Type,
                2 => ScriptHashType::Data1,
                _ => {
                    return Err(Error::WrongHex);
                }
            }
        };

        let witness_index = {
            let vec = decode(datas[2]).map_err(|_| Error::WrongHex)?;
            let array: [u8; 2] = vec.try_into().map_err(|_| Error::WrongHex)?;
            u16::from_le_bytes(array)
        };

        let script_args = {
            if datas[3].len() % 2 == 1 {
                return Err(Error::WrongHex);
            }
            match decode(datas[3]) {
                Err(_) => return Err(Error::WrongHex),
                Ok(v) => v,
            }
        };

        Ok(Self {
            code_hash: code_hash,
            hash_type,
            witness_index,
            script_args,
        })
    }

    pub fn to_str(self) -> Result<String, Error> {
        // check
        if self.script_args.len() > 32 * 1024 {
            return Err(Error::WrongHex);
        }

        // code_hash(fixed 32bytes) + hashtype + witness_index(max) + args + delimiter(:)
        let r_len = 64 + 2 + 4 + self.script_args.len() * 2 + 3;
        let mut data = Vec::<u8>::new();
        data.resize(r_len, 0);

        let mut offset = 0;

        // code_hash
        offset = Self::vec_to_str(self.code_hash.as_slice(), &mut data, offset);
        data[offset] = ':' as u8;
        offset += 1;

        // hash type
        data[offset] = '0' as u8;
        offset += 1;
        match self.hash_type {
            ScriptHashType::Data => data[offset] = '0' as u8,
            ScriptHashType::Type => data[offset] = '1' as u8,
            ScriptHashType::Data1 => data[offset] = '2' as u8,
        }
        data[offset + 1] = ':' as u8;
        offset += 2;

        // witness index
        offset = Self::vec_to_str(&self.witness_index.to_le_bytes(), &mut data, offset);
        data[offset] = ':' as u8;
        offset += 1;

        // args
        offset = Self::vec_to_str(&self.script_args, &mut data, offset);

        let r = String::from_utf8(data[..offset].to_vec());
        match r {
            Err(_) => return Err(Error::WrongHex),
            Ok(v) => Ok(v),
        }
    }

    #[inline]
    fn check_char(c: char) -> bool {
        c.eq(&(':'))
            || (c.ge(&('0')) && c.le(&('9')))
            || (c.ge(&('A')) && c.le(&('F')))
            || (c.ge(&('a')) && c.le(&('f')))
    }
    fn vec_to_str(d: &[u8], r: &mut [u8], offset: usize) -> usize {
        let d = encode_upper(d);
        let d_byte = d.as_bytes();
        let d_len = d_byte.len();
        r[offset..d_len + offset].copy_from_slice(d_byte);
        d_len + offset
    }
}

#[test]
fn test_child_script_entry_fmt() {
    let data =
        "11223344556677889900AABBCCDDEEFF11223344556677889900AABBCCDDEEFF:01:2A13:2312341231";
    let data2 = ChildScriptEntry::from_str(data);
    assert!(data2.is_ok());
    let data2 = data2.unwrap();

    assert_eq!(
        data2.code_hash.as_slice(),
        [
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0x00, 0xAA, 0xBB, 0xCC, 0xDD,
            0xEE, 0xFF, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0x00, 0xAA, 0xBB,
            0xCC, 0xDD, 0xEE, 0xFF,
        ]
    );
    assert!(data2.hash_type == ScriptHashType::Type);
    assert_eq!(data2.witness_index, 0x132A);
    assert_eq!(
        data2.script_args.to_vec().as_slice(),
        [0x23, 0x12, 0x34, 0x12, 0x31]
    );

    let data3 = data2.to_str().unwrap();

    assert_eq!(data3.as_str(), data);
}

#[test]
fn test_child_script_entry_from_str() {
    assert!(ChildScriptEntry::from_str(
        "223344556677889900AABBCCDDEEFF11223344556677889900AABBCCDDEEFF:01:2A13:2312341231"
    )
    .is_err());

    assert!(ChildScriptEntry::from_str(
        "1X223344556677889900AABBCCDDEEFF11223344556677889900AABBCCDDEEFF:01:2A13:2312341231"
    )
    .is_err());

    assert!(ChildScriptEntry::from_str(
        "11223344556677889900AABBCCDDEEFF11223344556677889900AABBCCDDEEFF:01:2A13"
    )
    .is_err());

    assert!(ChildScriptEntry::from_str(
        "11223344556677889900AABBCCDDEEFF11223344556677889900AABBCCDDEEFF:01:12A13:2312341231"
    )
    .is_err());

    assert!(ChildScriptEntry::from_str(
        "11223344556677889900AABBCCDDEEFF11223344556677889900AABBCCDDEEFF:11:2A13:2312341231"
    )
    .is_err());

    assert!(ChildScriptEntry::from_str(
        "1223344556677889900AABBCCDDEEFF11223344556677889900AABBCCDDEEFF:01:2A13:2312341231"
    )
    .is_err());
}

#[test]
fn test_child_script_entry_to_str() {
    assert_eq!(
        ChildScriptEntry {
            code_hash: [0u8; 32],
            hash_type: ScriptHashType::Data,
            witness_index: 0xFF11,
            script_args: [0x11].to_vec(),
        }
        .to_str()
        .unwrap(),
        "0000000000000000000000000000000000000000000000000000000000000000:00:11FF:11"
    );
}

#[test]
fn test_check_char() {
    assert_eq!(ChildScriptEntry::check_char('A'), true);
    assert_eq!(ChildScriptEntry::check_char('F'), true);
    assert_eq!(ChildScriptEntry::check_char('0'), true);
    assert_eq!(ChildScriptEntry::check_char('9'), true);
    assert_eq!(ChildScriptEntry::check_char('6'), true);
    assert_eq!(ChildScriptEntry::check_char('c'), true);
    assert_eq!(ChildScriptEntry::check_char('f'), true);
    assert_eq!(ChildScriptEntry::check_char('x'), false);
    assert_eq!(ChildScriptEntry::check_char('"'), false);
}

#[test]
fn test_vec_to_char() {
    let data = [0xaa, 0x21, 0x02];
    let mut buf = Vec::new();
    buf.resize(data.len() * 2, 0);
    let r = ChildScriptEntry::vec_to_str(&data, &mut buf, 0);
    assert_eq!(r, data.len() * 2);
    let buf = String::from_utf8(buf).unwrap();
    assert_eq!(buf.as_str(), "AA2102");

    let data = [0xaa, 0x21, 0x02];
    let mut buf = Vec::new();
    buf.resize(data.len() * 2 + 2, 0);
    buf[0] = '0' as u8;
    buf[1] = 'x' as u8;
    let r = ChildScriptEntry::vec_to_str(&data, &mut buf, 2);
    assert_eq!(r, data.len() * 2 + 2);
    let buf = String::from_utf8(buf).unwrap();
    assert_eq!(buf.as_str(), "0xAA2102")
}
