use std::{
    cmp::min,
    error::Error,
    fs::File,
    io::{self, Read, Seek},
    mem::MaybeUninit,
    string::FromUtf8Error,
};

use snap::raw::Decoder;

use crate::trace;

#[derive(Debug, Clone)]
pub struct Position {
    pub chunk_offset: usize,
    pub position_in_chunk: usize,
}

#[derive(Debug)]
pub enum SnappyError {
    Io(io::Error),
    InvalidHeader,
    DecompressionError(snap::Error),
    InsufficientData,
    ConversionError,
}

impl std::fmt::Display for SnappyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err = self.to_string();
        write!(f, "{}", err)
    }
}

impl Error for SnappyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SnappyError::DecompressionError(err) => Some(err),
            SnappyError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for SnappyError {
    fn from(value: io::Error) -> Self {
        SnappyError::Io(value)
    }
}

impl From<snap::Error> for SnappyError {
    fn from(value: snap::Error) -> Self {
        SnappyError::DecompressionError(value)
    }
}

impl From<FromUtf8Error> for SnappyError {
    fn from(value: FromUtf8Error) -> Self {
        let _ = value;
        SnappyError::ConversionError
    }
}
pub struct SnappyFile {
    snappy_file: File,
    snappy_decoder: Decoder,
    cache: Vec<u8>,
    cache_pos: usize,
    chunk_offset: usize,
}

impl SnappyFile {
    pub fn new(path: &str) -> Result<Self, SnappyError> {
        let mut snappy_file = File::open(path)?;
        let mut buffer: [u8; 2] = [0; 2];

        snappy_file.read_exact(&mut buffer).unwrap();
        if &buffer == b"at" {
            Ok(Self {
                snappy_file,
                snappy_decoder: snap::raw::Decoder::new(),
                cache: Vec::new(),
                cache_pos: 0,
                chunk_offset: 0,
            })
        } else {
            Err(SnappyError::InvalidHeader)
        }
    }
    fn ensure_cache_capacity(&mut self, size: usize) {
        if size > self.cache.capacity() {
            self.cache.reserve(size - self.cache.capacity());
        }
        self.cache_pos = 0;
    }
    fn read_compressed_length(&mut self) -> Result<usize, SnappyError> {
        let mut buffer = [0u8; 4];
        self.snappy_file.read_exact(&mut buffer)?;
        let chunk_len = u32::from_le_bytes(buffer) as usize;
        Ok(chunk_len)
    }
    fn load_next_chunk(&mut self) -> Result<(), SnappyError> {
        self.chunk_offset = self.snappy_file.stream_position()? as usize;
        let compressed_length = self.read_compressed_length()?;
        let mut buffer = vec![0u8; compressed_length];
        self.cache = match self.snappy_file.read_exact(&mut buffer) {
            Ok(_) => {
                let uncompressed_length = snap::raw::decompress_len(&buffer)?;
                self.ensure_cache_capacity(uncompressed_length);
                self.snappy_decoder.decompress_vec(&buffer)?
            }
            Err(err) => Err(err)?,
        };

        Ok(())
    }
    fn cache_remaining(&self) -> usize {
        self.cache.len().saturating_sub(self.cache_pos)
    }

    fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>, SnappyError> {
        let mut return_value = vec![0u8; len];
        if self.cache_remaining() >= len {
            return_value = self.cache[self.cache_pos..(len + self.cache_pos)].to_vec();
            self.cache_pos += len;
        } else {
            let mut size_to_read = len;
            while size_to_read > 0 {
                let chunk_size = min(self.cache.len() - self.cache_pos, size_to_read as usize);
                let offset = len - size_to_read;
                return_value[offset..offset + chunk_size]
                    .copy_from_slice(&self.cache[self.cache_pos..(chunk_size + self.cache_pos)]);
                self.cache_pos += chunk_size;
                size_to_read -= chunk_size;
                if size_to_read > 0 {
                    self.load_next_chunk()?;
                }
            }
        }
        Ok(return_value)
    }

    pub fn read_type<T>(&mut self) -> Result<T, SnappyError> {
        let mut tmp = MaybeUninit::<T>::uninit();
        let value = self.read_bytes(size_of::<T>())?;
        unsafe {
            std::ptr::copy_nonoverlapping(
                value.as_ptr(),
                tmp.as_mut_ptr() as *mut u8,
                size_of::<T>(),
            )
        };
        Ok(unsafe { tmp.assume_init() })
    }
    pub fn read_varint(&mut self) -> Result<usize, SnappyError> {
        let mut return_value: usize = 0;
        let mut shift: usize = 0;
        'parse: loop {
            let single_val = match self.read_type::<u8>() {
                Ok(val) => val,
                Err(_) => {
                    break 'parse;
                }
            };
            return_value |= (single_val as usize & 0x7f) << shift;
            shift += 7;
            if single_val & 0x80 == 0 {
                break 'parse;
            }
            if shift >= usize::BITS as usize {
                return Err(SnappyError::InsufficientData);
            }
        }
        Ok(return_value)
    }
    pub fn read_string(&mut self) -> Result<String, SnappyError> {
        let len = self.read_varint()?;
        if len == 0 {
            return Err(SnappyError::InsufficientData);
        }
        let buffer = self.read_bytes(len)?;
        Ok(String::from_utf8(buffer)?)
    }
    pub fn get_current_offset(&mut self) -> Position {
        Position {
            chunk_offset: self.chunk_offset,
            position_in_chunk: self.cache_pos,
        }
    }
    pub fn read_signed_varint(&mut self) -> Result<i64, SnappyError> {
        match self.read_type::<u8>() {
            Ok(val) => match val {
                n if trace::Type::TypeSint as u8 == n => Ok(-(self.read_varint()? as i64)),
                n if trace::Type::TypeUint as u8 == n => Ok(self.read_varint()? as i64),
                _=> panic!("Unexpected type")
            },
            Err(_) => Ok(0i64)
        }
    }
}
