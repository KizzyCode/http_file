use std;
use super::error::{Error, ErrorType};
use super::http;

fn http_request_response(request: http::RequestHeader, connection: &mut super::Connection, timeout_point: std::time::Instant) -> Result<http::ResponseHeader, std::io::Error> {
	use http::{WriteableHeader, ReadableHeader};
	
	// Send request-header
	let mut request = request.into_writer();
	request.write(connection, super::time_remaining(timeout_point))?;
	
	// Receive response-header
	let mut response = http::ResponseHeaderReader::new(8192);
	response.read(connection, super::time_remaining(timeout_point))?;
	
	// Parse response-header
	http::ResponseHeader::from_reader(response)
}

fn http_response_body(buffer: &mut[u8], connection: &mut super::Connection, timeout_point: std::time::Instant) -> Result<(), std::io::Error> {
	use http::ReadableBody;
	
	// Receive response-body
	let mut response = http::SizedBodyReader::new(buffer.len() as u64);
	if !response.read(buffer, &mut 0, connection, super::time_remaining(timeout_point))? { return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof)) }
	Ok(())
}



pub fn receive_size(uri: &super::URI, connection: &mut super::Connection, timeout: std::time::Duration) -> Result<u64, Error> {
	let timeout_point = std::time::Instant::now() + timeout;
	
	// Build HTTP-request
	let mut request = http::RequestHeader::default();
	request.http_method = "HEAD".to_owned();
	request.request_uri = uri.resource.clone();
	request.header_fields.insert("Host".to_owned(), uri.server.clone());
	request.header_fields.insert("Content-Length".to_owned(), "0".to_owned());
	request.header_fields.insert("Connection".to_owned(), "keep-alive".to_owned());
	
	// Try to send HTTP-request
	'retry_loop: while super::time_remaining(timeout_point) > std::time::Duration::default() {
		// Send request-header and receive response-header
		let response = match http_request_response(request.clone(), connection, timeout_point) {
			Ok(response) => response,
			Err(ref error) if super::Connection::is_recoverable(error) => {
				connection.reconnect(super::time_remaining(timeout_point))?;
				continue 'retry_loop
			},
			Err(error) => throw_err!(ErrorType::from(error))
		};
		
		// Parse response
		if response.http_status_code_reason.0 != 200 { throw_err!(ErrorType::IOAccessError, format!("HTTP-error {}: {}", response.http_status_code_reason.0, &response.http_status_code_reason.1)) }
		
		let accept_ranges = if let Some(accept_ranges) = response.header_fields.get("Accept-Ranges") { accept_ranges }
			else { throw_err!(ErrorType::Unsupported, "The server does not support partial-content-requests".to_owned()) };
		if accept_ranges != "bytes" { throw_err!(ErrorType::Unsupported, "The server does not support byte-indexed partial-content-requests".to_owned()) }
		
		let length_field = if let Some(length_field) = response.header_fields.get("Content-Length") { length_field }
			else { throw_err!(ErrorType::Unsupported, "The server did not send a \"Content-Length\"-field".to_owned()) };
		let length = try_err!(length_field.parse::<u64>(), "The server returned an invalid \"Content-Length\"-field".to_owned());
		
		return Ok(length)
	};
	throw_err!(ErrorType::from(std::io::Error::from(std::io::ErrorKind::TimedOut)))
}



pub fn receive_chunk(uri: &super::URI, connection: &mut super::Connection, buffer: &mut[u8], file_offset: u64, timeout: std::time::Duration) -> Result<(), Error> {
	let timeout_point = std::time::Instant::now() + timeout;
	
	// Check buffer-length
	if buffer.len() == 0 { return Ok(()) }
	
	// Build HTTP-request
	let mut request = http::RequestHeader::default();
	request.http_method = "GET".to_owned();
	request.request_uri = uri.resource.clone();
	request.header_fields.insert("Host".to_owned(), uri.server.clone());
	request.header_fields.insert("Content-Length".to_owned(), "0".to_owned());
	request.header_fields.insert("Connection".to_owned(), "keep-alive".to_owned());
	request.header_fields.insert("Range".to_owned(), format!("bytes={}-{}", file_offset, (file_offset + buffer.len() as u64) - 1));
	
	// Try to send HTTP-request and receive the response-header and -body `retries`-times
	'retry_loop: while super::time_remaining(timeout_point) > std::time::Duration::default() {
		// Send request-header and receive response-header
		let response = match http_request_response(request.clone(), connection, timeout_point) {
			Ok(response) => response,
			Err(ref error) if super::Connection::is_recoverable(error) => {
				connection.reconnect(super::time_remaining(timeout_point))?;
				continue 'retry_loop
			},
			Err(error) => {
				let description = format!("{:?}", &error);
				throw_err!(ErrorType::from(error), description)
			}
		};
		
		// Parse response
		if response.http_status_code_reason.0 != 206 { throw_err!(ErrorType::IOAccessError, format!("HTTP-error {}: {}", response.http_status_code_reason.0, &response.http_status_code_reason.1)) }
		
		let range = if let Some(accept_ranges) = response.header_fields.get("Content-Range") { accept_ranges }
			else { throw_err!(ErrorType::Unsupported, "The server did not respond with a chunk".to_owned()) };
		let served_range = super::substring_until_pattern(&range, &mut 0, "/", true)?;
		if served_range != format!("bytes {}-{}", file_offset, (file_offset + buffer.len() as u64) - 1) { throw_err!(ErrorType::InvalidData, "The server send a chunk with unexpected length".to_owned()) }
		
		// Receive response-body
		match http_response_body(buffer, connection, timeout_point) {
			Ok(_) => return Ok(()),
			Err(ref error) if super::Connection::is_recoverable(error) => {
				connection.reconnect(super::time_remaining(timeout_point))?;
				continue 'retry_loop
			},
			Err(error) => throw_err!(ErrorType::from(error))
		}
	};
	throw_err!(ErrorType::from(std::io::Error::from(std::io::ErrorKind::TimedOut)))
}