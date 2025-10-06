use std::{
    fs::File,
    io::{BufWriter, Write as _},
    path::Path,
};

const CHUNK_SIZE: usize = 64 * 1024;
const BUFFER_SIZE: usize = 1024 * 1024;

pub struct StreamingDatasetWriter {
    writer: BufWriter<File>,
    chunk_buffer: Box<[u8; CHUNK_SIZE]>,
    bytes_written: u64,
}

impl StreamingDatasetWriter {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let file = File::create(path)?;
        let writer = BufWriter::with_capacity(BUFFER_SIZE, file);

        Ok(Self {
            writer,
            chunk_buffer: vec![0u8; CHUNK_SIZE]
                .into_boxed_slice()
                .try_into()
                .expect("CHUNK_SIZE is const"),
            bytes_written: 0,
        })
    }

    pub fn write_chunk(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        let mut remaining = data;

        while !remaining.is_empty() {
            let write_size = std::cmp::min(remaining.len(), CHUNK_SIZE);
            self.chunk_buffer[..write_size].copy_from_slice(&remaining[..write_size]);
            self.writer.write_all(&self.chunk_buffer[..write_size])?;
            remaining = &remaining[write_size..];
            self.bytes_written += write_size as u64;
        }

        Ok(())
    }

    pub fn finalize(mut self) -> Result<u64, std::io::Error> {
        self.writer.flush()?;
        Ok(self.bytes_written)
    }
}
