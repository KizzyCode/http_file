use std;
use super::error::{Error, ErrorType};
extern crate network_io;
extern crate http;

pub struct Connection {
	address: std::net::SocketAddr,
	stream: network_io::TcpStream
}
impl Connection {
	pub fn connect(address: &super::URI, timeout: std::time::Duration) -> Result<Self, Error> {
		let timeout_point = std::time::Instant::now() + timeout;
		
		// Try to resolve and connect until the timeout is reached
		'retry_loop: while super::time_remaining(timeout_point) > std::time::Duration::default() {
			// Resolve address
			let address = match network_io::resolver::resolve_address(&address.server, super::time_remaining(timeout_point)) {
				Ok(address) => address,
				Err(ref error) if Connection::is_recoverable(error) => continue 'retry_loop,
				Err(error) => throw_err!(ErrorType::from(error))
			};
			
			// Connect TCP-stream
			let stream = match network_io::TcpStream::connect(address, super::time_remaining(timeout_point)) {
				Ok(stream) => stream,
				Err(ref error) if Connection::is_recoverable(error) => continue 'retry_loop,
				Err(error) => throw_err!(ErrorType::from(error))
			};
			return Ok(Connection{ address, stream })
		}
		throw_err!(ErrorType::from(std::io::Error::from(std::io::ErrorKind::TimedOut)))
	}
	
	pub fn reconnect(&mut self, timeout: std::time::Duration) -> Result<(), Error> {
		let timeout_point = std::time::Instant::now() + timeout;
		
		// Try to shutdown the socket
		let _ = self.stream.shutdown(std::net::Shutdown::Both);
		
		// Try to reconnect
		'retry_loop: while super::time_remaining(timeout_point) > std::time::Duration::default() {
			self.stream = match network_io::TcpStream::connect(self.address, timeout) {
				Ok(stream) => stream,
				Err(ref error) if Connection::is_recoverable(error) => continue 'retry_loop,
				Err(error) => throw_err!(ErrorType::from(error))
			};
			return Ok(())
		}
		throw_err!(ErrorType::from(std::io::Error::from(std::io::ErrorKind::TimedOut)))
	}
	
	pub fn is_recoverable(error: &std::io::Error) -> bool {
		match error.kind() {
			std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::ConnectionAborted | std::io::ErrorKind::BrokenPipe | std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut | std::io::ErrorKind::UnexpectedEof => true,
			_ => false
		}
	}
}

impl http::ReadableStream for Connection {
	fn read(&mut self, buffer: &mut[u8], buffer_pos: &mut usize, timeout: std::time::Duration) -> Result<(), std::io::Error> {
		self.stream.read(buffer, buffer_pos, timeout)
	}
	fn read_until(&mut self, buffer: &mut[u8], buffer_pos: &mut usize, pattern: &[u8], timeout: std::time::Duration) -> Result<(), std::io::Error> {
		self.stream.read_until(buffer, buffer_pos, pattern, timeout)
	}
}
impl http::WriteableStream for Connection {
	fn write(&mut self, data: &[u8], data_pos: &mut usize, timeout: std::time::Duration) -> Result<(), std::io::Error> {
		self.stream.write(data, data_pos, timeout)
	}
}