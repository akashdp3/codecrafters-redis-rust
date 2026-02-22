use anyhow::Ok;
use bytes::Bytes;
use std::{collections::HashMap, path::PathBuf};
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
};

#[derive(Debug)]
pub(crate) struct RDB {
    header: String,
    metadata: HashMap<String, String>,
    pub data: HashMap<String, String>,
}

impl RDB {
    pub(crate) fn new() -> Self {
        Self {
            header: "".to_string(),
            metadata: HashMap::new(),
            data: HashMap::new(),
        }
    }

    pub(crate) fn header(&mut self, msg: &str) -> () {
        self.header = msg.to_string();
    }

    pub(crate) fn metadata(&mut self, key: &str, val: &str) -> () {
        self.metadata.insert(key.to_string(), val.to_string());
    }

    pub(crate) fn data(&mut self, key: &str, val: &str) -> () {
        self.data.insert(key.to_string(), val.to_string());
    }
}

enum LengthEncoding {
    Length(usize),
    Integer(i64),
}

pub(crate) struct RDBParser {
    reader: BufReader<File>,
}

impl RDBParser {
    pub(crate) async fn new(path: PathBuf) -> anyhow::Result<Self> {
        let file = File::open(path).await?;

        Ok(Self {
            reader: BufReader::new(file),
        })
    }

    async fn read_exact(&mut self, len: usize) -> anyhow::Result<Bytes> {
        let mut buf = vec![0; len];

        self.reader.read_exact(&mut buf).await?;
        Ok(Bytes::from(buf))
    }

    async fn read_byte(&mut self) -> anyhow::Result<Bytes> {
        self.read_exact(1).await
    }

    async fn read_length_or_int(&mut self) -> anyhow::Result<LengthEncoding> {
        let first = self.read_byte().await?[0];
        let enc_type = (first & 0xC0) >> 6; // Top 2 bits

        match enc_type {
            0 => Ok(LengthEncoding::Length((first & 0x3F) as usize)),
            1 => {
                let second = self.read_byte().await?[0];
                let len = (((first & 0x3F) as usize) << 8) | (second as usize);
                Ok(LengthEncoding::Length(len))
            }
            2 => {
                let bytes = self.read_exact(4).await?;
                let len = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
                Ok(LengthEncoding::Length(len))
            }
            3 => {
                // Special encoding - integer stored as string
                match first & 0x3F {
                    0 => {
                        let val = self.read_byte().await?[0] as i8;
                        Ok(LengthEncoding::Integer(val as i64))
                    }
                    1 => {
                        let bytes = self.read_exact(2).await?;
                        let val = i16::from_le_bytes([bytes[0], bytes[1]]);
                        Ok(LengthEncoding::Integer(val as i64))
                    }
                    2 => {
                        let bytes = self.read_exact(4).await?;
                        let val = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                        Ok(LengthEncoding::Integer(val as i64))
                    }
                    _ => anyhow::bail!("Unknown special encoding"),
                }
            }
            _ => unreachable!(),
        }
    }

    async fn read_string(&mut self) -> anyhow::Result<String> {
        match self.read_length_or_int().await? {
            LengthEncoding::Length(len) => {
                let bytes = self.read_exact(len).await?;
                Ok(String::from_utf8(bytes.to_vec())?)
            }
            LengthEncoding::Integer(val) => Ok(val.to_string()),
        }
    }

    pub(crate) async fn parse(&mut self) -> anyhow::Result<RDB> {
        let header = self.read_exact(9).await?;
        let header = String::from_utf8(header.to_vec())?;

        let mut rdb = RDB::new();
        rdb.header(&header);

        loop {
            let next = self.read_byte().await?;

            match next[0] {
                0xFA => {
                    let key = self.read_string().await?;
                    let val = self.read_string().await?;

                    rdb.metadata(&key, &val);
                }
                0xFE => {
                    let _db_index = self.read_byte().await?;

                    let _ = self.read_byte().await?;

                    let total_keys = self.read_byte().await?[0] as usize;
                    let _expire_keys = self.read_byte().await?[0] as usize;

                    for _ in 0..total_keys {
                        let _data_index = self.read_byte().await?;

                        let key = self.read_string().await?;
                        let val = self.read_string().await?;

                        rdb.data(&key, &val);
                    }
                }
                0xFF => {
                    break;
                }
                x => {
                    anyhow::bail!(format!("Unexpected byte encountered!!! Byte: {}", x));
                }
            }
        }

        Ok(rdb)
    }
}
