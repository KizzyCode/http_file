extern crate http;
extern crate network_io;

#[macro_use] pub mod error;
mod uri;
mod connection;
mod http_action;
mod cache;
pub mod file;

use uri::URI;
use connection::Connection;
use cache::CacheDB;

pub use error::{Error, ErrorType};
pub use file::File;



// Helper functions
fn substring_until_pattern(string: &str, string_byte_offset: &mut usize, separator: &str, strip_separator: bool) -> Result<String, Error> {
	// Get substring starting at `string_byte_offset`
	if *string_byte_offset >= string.len() { throw_err!(ErrorType::InvalidParameter) };
	let string = string.split_at(*string_byte_offset).1;
	
	// Find first occurrence of `separator`
	let pos = if let Some(pos) = string.find(separator) { pos }
		else { throw_err!(ErrorType::InvalidParameter) };
	
	// Translate `pos` into a byte-offset and adjust
	let offset = string.char_indices().skip(pos).next().unwrap().0;
	
	// Get substring
	let extracted = string.split_at(offset).0.to_owned();
	*string_byte_offset += offset + if strip_separator { separator.len() }
		else { 0 };
	Ok(extracted)
}

fn time_remaining(timeout_point: std::time::Instant) -> std::time::Duration {
	http::time_remaining(timeout_point)
}