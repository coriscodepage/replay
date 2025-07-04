use std::{
    cmp::min,
    error::Error,
    fs::File,
    io::{self, Read, Seek},
    mem::MaybeUninit,
    panic::Location,
    string::FromUtf8Error,
};

use snap::raw::Decoder;

use crate::trace;

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub struct Position {
    pub chunk_offset: usize,
    pub position_in_chunk: usize,
}

#[derive(Debug)]
pub enum SnappyError {
    Io(&'static Location<'static>, io::Error),
    InvalidHeader(&'static Location<'static>),
    DecompressionError(&'static Location<'static>, snap::Error),
    InsufficientData(&'static Location<'static>),
    ConversionError(&'static Location<'static>, String),
}

impl SnappyError {
    #[track_caller]
    pub fn io_error(error: io::Error) -> Self {
        Self::Io(Location::caller(), error)
    }

    #[track_caller]
    pub fn invalid_header() -> Self {
        Self::InvalidHeader(Location::caller())
    }

    #[track_caller]
    pub fn decompression_error(error: snap::Error) -> Self {
        Self::DecompressionError(Location::caller(), error)
    }

    #[track_caller]
    pub fn insufficient_data() -> Self {
        Self::InsufficientData(Location::caller())
    }

    #[track_caller]
    pub fn conversion_error(message: String) -> Self {
        Self::ConversionError(Location::caller(), message)
    }
}

impl std::fmt::Display for SnappyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SnappyError::Io(location, err) => {
                write!(f, "IO error: {} at {}:{}", err, location.file(), location.line())
            }
            SnappyError::InvalidHeader(location) => {
                write!(f, "Invalid header at {}:{}", location.file(), location.line())
            }
            SnappyError::DecompressionError(location, err) => {
                write!(
                    f,
                    "Decompression error: {} at {}:{}",
                    err,
                    location.file(),
                    location.line()
                )
            }
            SnappyError::InsufficientData(location) => {
                write!(f, "Insufficient data at {}:{}", location.file(), location.line())
            }
            SnappyError::ConversionError(location, msg) => {
                write!(
                    f,
                    "Conversion error: {} at {}:{}",
                    msg,
                    location.file(),
                    location.line()
                )
            }
        }
    }
}

impl Error for SnappyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SnappyError::DecompressionError(_, err) => Some(err),
            SnappyError::Io(_, err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for SnappyError {
    fn from(value: io::Error) -> Self {
        SnappyError::io_error(value)
    }
}

impl From<snap::Error> for SnappyError {
    fn from(value: snap::Error) -> Self {
        SnappyError::decompression_error(value)
    }
}

impl From<FromUtf8Error> for SnappyError {
    fn from(value: FromUtf8Error) -> Self {
        SnappyError::conversion_error(value.to_string())
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
            Err(SnappyError::invalid_header())
        }
    }
    fn ensure_cache_capacity(&mut self, size: usize) {
        if size > self.cache.capacity() {
            self.cache.resize(size, 0);
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
        match self.snappy_file.read_exact(&mut buffer) {
            Ok(_) => {
                let uncompressed_length = snap::raw::decompress_len(&buffer)?;
                self.ensure_cache_capacity(uncompressed_length);
                self.snappy_decoder.decompress(&buffer, &mut self.cache)?
            }
            Err(err) => Err(err)?,
        };

        Ok(())
    }
    fn cache_remaining(&self) -> usize {
        self.cache.len().saturating_sub(self.cache_pos)
    }

    pub fn read_bytes(&mut self, buf: &mut [u8]) -> Result<(), SnappyError> {
        let len = buf.len();
        if self.cache_remaining() >= len {
            buf.copy_from_slice(&self.cache[self.cache_pos..(len + self.cache_pos)]);
            self.cache_pos += len;
        } else {
            let mut size_to_read = len;
            while size_to_read > 0 {
                let chunk_size = min(self.cache.len() - self.cache_pos, size_to_read as usize);
                let offset = len - size_to_read;
                buf[offset..offset + chunk_size]
                    .copy_from_slice(&self.cache[self.cache_pos..(chunk_size + self.cache_pos)]);
                self.cache_pos += chunk_size;
                size_to_read -= chunk_size;
                if size_to_read > 0 {
                    self.load_next_chunk()?;
                }
            }
        }
        Ok(())
    }

    pub fn read_type<T: Sized>(&mut self) -> Result<T, SnappyError> {
        let mut tmp = MaybeUninit::<T>::uninit();
        let mut buffer = vec![0u8; size_of::<T>()];
        self.read_bytes(&mut buffer)?;
        unsafe {
            std::ptr::copy_nonoverlapping(
                buffer.as_ptr(),
                tmp.as_mut_ptr() as *mut u8,
                size_of::<T>(),
            );
        }
        Ok(unsafe { tmp.assume_init() })
    }
    pub fn read_varint(&mut self) -> Result<usize, SnappyError> {
        let mut return_value: usize = 0;
        let mut shift: usize = 0;
        let mut single_val = [0u8; 1];
        'parse: loop {
            match self.read_bytes(&mut single_val) {
                Ok(_) => {
                    let single_val = single_val[0];
                    return_value |= (single_val as usize & 0x7f) << shift;
                    shift += 7;
                    if single_val & 0x80 == 0 {
                        break 'parse;
                    }
                    if shift >= usize::BITS as usize {
                        return Err(SnappyError::insufficient_data());
                    }
                },
                Err(_) => {
                    break 'parse;
                },
            };
        }
        Ok(return_value)
    }
    pub fn read_string(&mut self) -> Result<String, SnappyError> {
        let len = self.read_varint()?;
        if len == 0 {
            return Err(SnappyError::insufficient_data());
        }
        let mut buffer = vec![0u8; len];
        self.read_bytes(&mut buffer)?;
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
                _ => panic!("Unexpected type"),
            },
            Err(_) => Ok(0i64),
        }
    }
}
