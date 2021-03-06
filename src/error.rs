use std;

#[derive(Debug)]
/// The error-type
pub enum ErrorType {
	/// Invalid data (invalid encoding, integrity error etc.)
	InvalidData,
	/// Not enough resources to process data
	ResourceError,
	
	/// Failed to access element
	IOAccessError,
	/// Failed to read from or write to element
	IOReadWriteError,
	
	/// Other IO-error
	GenericIOError(std::io::Error),
	
	/// Invalid parameter (not in range, does not make sense etc.)
	InvalidParameter,
	/// The parameter might be valid but us unsupported
	Unsupported,
	
	/// Another error
	Other(String)
}
impl From<std::io::Error> for ErrorType {
	fn from(error: std::io::Error) -> Self {
		match error.kind() {
			std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied => ErrorType::IOAccessError,
			std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::ConnectionAborted | std::io::ErrorKind::BrokenPipe | std::io::ErrorKind::UnexpectedEof => ErrorType::IOReadWriteError,
			_ => ErrorType::GenericIOError(error)
		}
	}
}
impl From<std::str::Utf8Error> for ErrorType {
	fn from(_: std::str::Utf8Error) -> Self {
		ErrorType::InvalidData
	}
}
impl From<std::num::ParseIntError> for ErrorType {
	fn from(_: std::num::ParseIntError) -> Self {
		ErrorType::InvalidData
	}
}



#[derive(Debug)]
/// An error-describing structure containing the error and it's file/line
pub struct Error {
	/// The error-type
	pub error_type: ErrorType,
	/// Description
	pub description: String,
	/// The file in which the error occurred
	pub file: &'static str,
	/// The line on which the error occurred
	pub line: u32
}

#[macro_export]
/// Create an error from an `ErrorType`
macro_rules! new_err {
	($error_type:expr, $description:expr) => (Err($crate::error::Error {
		error_type: $error_type,
		description: $description,
		file: file!(),
		line: line!()
	}));
	($error_type:expr) => (new_err!($error_type, "".to_owned()));
}

#[macro_export]
/// Create an error from an `ErrorType`
macro_rules! throw_err {
	($error_type:expr, $description:expr) => (return new_err!($error_type, $description));
	($error_type:expr) => (throw_err!($error_type, "".to_owned()));
}

#[macro_export]
/// Tries an expression and propagates an eventual error
macro_rules! try_err {
	($code:expr, $description:expr) => (match $code {
		Ok(result) => result,
		Err(error) => throw_err!($crate::error::ErrorType::from(error), $description)
	});
	($code:expr) => (try_err!($code, "".to_owned()))
}