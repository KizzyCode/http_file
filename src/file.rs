use std;
use super::error::{Error, ErrorType};

static DEFAULT_CACHE_CHUNK_SIZE: usize = 131_072;
static DEFAULT_CACHE_CHUNK_COUNT: usize = 2048;

pub struct File {
	size: u64,
	position: u64,
	
	uri: super::URI,
	connection: super::Connection,
	
	chunk_buf: Vec<u8>,
	cache: super::CacheDB
}
impl File {
	/// Opens a URI
	///
	/// _Note: If a recoverable error happens this function will retry the operation until the
	/// `timeout` exceeded. Recoverable errors are:_
	///
	///  - `ConnectionReset`
	///  - `ConnectionAborted`
	///  - `BrokenPipe`
	///  - `WouldBlock`
	///  - `TimedOut` (this applies only to internal timeouts)
	pub fn open(uri: &str, timeout: std::time::Duration) -> Result<Self, Error> {
		let timeout_point = std::time::Instant::now() + timeout;
		
		// Parse URI and connect to server
		let uri = super::URI::parse(uri)?;
		let mut connection = super::Connection::connect(&uri, super::time_remaining(timeout_point))?;
		
		// Receive file-size
		let size = super::http_action::receive_size(&uri, &mut connection, super::time_remaining(timeout_point))?;
		
		Ok(File {
			size, position: 0,
			uri, connection,
			chunk_buf: vec![0u8; DEFAULT_CACHE_CHUNK_SIZE], cache: super::CacheDB::new(DEFAULT_CACHE_CHUNK_COUNT, DEFAULT_CACHE_CHUNK_SIZE)
		})
	}
	
	/// Adjusts the cache-parameters
	///
	/// _Note: Changing these parameters will discard all cached chunks_
	pub fn adjust_cache_size(&mut self, chunk_count: usize, chunk_size: usize) {
		self.chunk_buf = vec![0u8; chunk_size];
		self.cache = super::CacheDB::new(chunk_count, chunk_size)
	}
	
	/// Returns the file-size
	pub fn size(&self) -> u64 {
		self.size
	}
	
	/// Returns the file-name
	pub fn name(&self) -> &str {
		&self.uri.name
	}
	
	/// Returns the current file-position
	pub fn tell(&self) -> u64 {
		self.position
	}
	
	/// Adjusts the current file-position
	///
	/// If `relative` is `true` the file-position will be adjusted relatively to the current file-
	/// position
	pub fn seek(&mut self, by: i64, relative: bool) -> Result<(), Error> {
		// Prepare values
		let position = if relative { self.position }
			else { 0 };
		let by_u64 = by.abs() as u64;
		
		// Check if we need to increment or decrement the counter
		if by > 0 {
			// Validate boundaries
			if position + by_u64 > self.size { throw_err!(ErrorType::InvalidParameter, format!("Cannot seek beyond EOF ({})", self.size)) }
			self.position = position + by_u64;
		} else if by < 0 {
			// Validate position
			if by_u64 > position { throw_err!(ErrorType::InvalidParameter, "Cannot seek before 0".to_owned()) }
			self.position = position - by_u64;
		}
		Ok(())
	}
	
	/// Reads `buffer.len()` bytes into `buffer` and returns either the amount of bytes read or an
	/// error
	///
	/// _Note: if the amount of bytes read is smaller than `buffer.len()` this ALWAYS means that the
	/// `EOF` was reached. Otherwise an error would be returned._
	pub fn read(&mut self, buffer: &mut[u8], timeout: std::time::Duration) -> Result<usize, Error> {
		// Read bytes and increment position
		let position = self.position;
		let result = self.read_at(buffer, position, timeout);
		match result {
			Ok(bytes_read) => { self.position += bytes_read as u64; Ok(bytes_read) },
			Err(error) => Err(error)
		}
	}
	
	/// Reads `buffer.len()` bytes into `buffer` beginning at `offset` (relative to `0`)
	/// and returns either the amount of bytes read or an error. This function does not modify the
	/// file-position.
	///
	/// _Note: if the amount of bytes read is smaller than `buffer.len()` this ALWAYS means that the
	/// `EOF` was reached. Otherwise an error would be returned._
	pub fn read_at(&mut self, buffer: &mut[u8], offset: u64, timeout: std::time::Duration) -> Result<usize, Error> {
		// Compute the amount of bytes to read
		let to_read = std::cmp::min(buffer.len(), (self.size - offset) as usize);
		
		// Read bytes
		self.read_range(&mut buffer[..to_read], offset, timeout)?;
		Ok(to_read)
	}
	
	
	
	fn read_range(&mut self, buffer: &mut[u8], offset: u64, timeout: std::time::Duration) -> Result<(), Error> {
		// Compute the aligned boundaries
		let aligned_offset = (offset / self.chunk_buf.len() as u64) * self.chunk_buf.len() as u64;
		let skip_left = (offset - aligned_offset) as usize;
		let chunk_count = ((skip_left + buffer.len()) / self.chunk_buf.len()) + 1;
		
		// Read data
		let (mut buffer_pos, chunk_size) = (0, self.chunk_buf.len());
		
		// Copy first partial block
		{
			// Fetch chunk
			self.read_chunk(aligned_offset, timeout)?;
			
			// Copy chunk
			let to_copy = std::cmp::min(self.chunk_buf.len() - skip_left, buffer.len() - buffer_pos);
			&mut buffer[buffer_pos..buffer_pos + to_copy].copy_from_slice(&self.chunk_buf[skip_left..skip_left + to_copy]);
			buffer_pos += to_copy
		}
		
		// Copy remaining chunks
		for i in 1..chunk_count {
			// Fetch chunk
			self.read_chunk(aligned_offset + (i * chunk_size) as u64, timeout)?;
			
			// Copy chunk
			let to_copy = std::cmp::min(self.chunk_buf.len(), buffer.len() - buffer_pos);
			&mut buffer[buffer_pos..buffer_pos + to_copy].copy_from_slice(&self.chunk_buf[.. to_copy]);
			buffer_pos += to_copy
		}
		Ok(())
	}
	
	fn read_chunk(&mut self, aligned_offset: u64, timeout: std::time::Duration) -> Result<(), Error> {
		// Compute chunk-size (necessary because the last chunk might be smaller than the usual chunk-size)
		let chunk_size = std::cmp::min((self.size - aligned_offset) as usize, self.chunk_buf.len());
		
		// Check if we have the chunk or if we neet to fetch the chunk
		if self.cache.contains(aligned_offset) {
			self.chunk_buf.copy_from_slice(self.cache.get(aligned_offset));
			Ok(())
		} else {
			super::http_action::receive_chunk(&self.uri, &mut self.connection, &mut self.chunk_buf[..chunk_size], aligned_offset, timeout)?;
			self.cache.insert(&self.chunk_buf, aligned_offset);
			Ok(())
		}
	}
}