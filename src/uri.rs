use super::error::{Error, ErrorType};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct URI {
	pub protocol: String,
	pub server: String,
	pub resource: String,
	pub name: String
}
impl URI {
	pub fn parse(uri_string: &str) -> Result<Self, Error> {
		use std::iter::FromIterator;
		
		let mut string_byte_offset = 0usize;
		
		// Parse URI
		let protocol = super::substring_until_pattern(uri_string, &mut string_byte_offset, "://", true)?;
		let mut server = super::substring_until_pattern(uri_string, &mut string_byte_offset, "/", false)?;
		let resource = uri_string.split_at(string_byte_offset).1.to_owned();
		
		// Validate protocols
		match protocol.as_str() {
			"http" | "https" => (),
			protocol => throw_err!(ErrorType::InvalidParameter, format!("The protocol \"{}\" is not supported", protocol))
		}
		
		// Check if the server ends with a port
		let (mut has_port, nums) = (false, ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']);
		for c in server.chars().rev() {
			if c == ':' { has_port = true; break; }
			if !nums.contains(&c) { break; }
		}
		
		// Append default-port if necessary
		if !has_port {
			server += match protocol.as_str() {
				"http" => ":80",
				"https" => ":443",
				_ => panic!("Should never occur because we have checked the protocols above")
			}
		}
		
		// Extract filename
		let uri_string_reversed = String::from_iter(uri_string.chars().rev());
		let name_reversed = super::substring_until_pattern(&uri_string_reversed, &mut 0, "/", true)?;
		
		let mut name = String::from_iter(name_reversed.chars().rev());
		if name.len() > 0 { name = URI::uri_decode(&name) }
			else { name += "UNNAMED" }
		
		Ok(URI{ protocol, server, resource, name })
	}
	
	fn uri_decode(to_decode: &str) -> String {
		// Copy string to byte-vector to replace
		let mut decoded = String::new();
		
		// Decoder-state
		enum State {
			Char, Percent1, Percent2(char)
		};
		let mut state = State::Char;
		
		// Decode-bytes
		'decode_loop: for c in to_decode.chars() {
			match state {
				// Expect any char
				State::Char => {
					// Check for percent-char
					if c == '%' { state = State::Percent1 }
						else { decoded.push(c) }
				},
				// Add the higher-percent-encoded-nibble
				State::Percent1 => state = State::Percent2(c),
				// Decode the percent-encoded nibbles
				State::Percent2(previous) => {
					state = State::Char;
					
					// Build string
					let mut hex_str = String::new();
					hex_str.push(previous);
					hex_str.push(c);
					
					// Decode hexes
					match u8::from_str_radix(&hex_str, 16) {
						Ok(value) => decoded.push(char::from(value)),
						Err(_) => decoded.push('�')
					}
				}
			}
		}
		
		decoded
	}
}