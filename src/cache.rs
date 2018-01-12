use std;
use std::collections::{BTreeMap, HashMap};

struct Entry {
	pub data: Vec<u8>,
	pub offset: u64,
	pub timestamp: u64
}

pub struct CacheDB {
	time_null: std::time::Instant,
	entries: Vec<Entry>,
	accessed: BTreeMap<u64, usize>,
	offset: HashMap<u64, usize>
}
impl CacheDB {
	pub fn new(chunk_count: usize, chunk_size: usize) -> Self {
		// Create and initialize entries and accessed-map
		let (mut entries, mut accessed) = (Vec::with_capacity(chunk_count), BTreeMap::new());
		for i in 0..chunk_count {
			// Allocate chunk
			let mut chunk = vec![0u8; chunk_size];
			chunk.reserve_exact(chunk_size);
			
			// Insert entries
			entries.push(Entry{ data: chunk, offset: std::u64::MAX, timestamp: i as u64 });
			accessed.insert(i as u64, i);
		}
		
		CacheDB{ time_null: std::time::Instant::now(), entries, accessed, offset: HashMap::new() }
	}
	
	fn update_timestamp(&mut self, index: usize) {
		let entry: &Entry = self.entries.get(index).unwrap();
		
		// Update entry in `accessed`
		self.accessed.remove(&entry.timestamp);
		
		// Make sure we don't overwrite a cache-entry
		loop {
			let timestamp = {
				let timestamp = self.time_null.elapsed();
				(timestamp.as_secs() * 1_000_000_000) + timestamp.subsec_nanos() as u64
			};
			if !self.accessed.contains_key(&timestamp) {
				self.accessed.insert(timestamp, index);
				break
			}
		}
	}
	
	pub fn insert(&mut self, data: &[u8], offset: u64) {
		// Get oldest entry and remove it from `offset`
		let oldest_entry = *self.accessed.iter().next().unwrap().1;
		self.offset.remove(&self.entries[oldest_entry].offset);
		
		// Update oldest entry
		&(self.entries[oldest_entry].data)[.. data.len()].copy_from_slice(data);
		self.entries[oldest_entry].offset = offset;
		
		// Reinsert/update entry
		self.offset.insert(offset, oldest_entry);
		self.update_timestamp(oldest_entry);
	}
	
	pub fn contains(&self, offset: u64) -> bool {
		self.offset.contains_key(&offset)
	}
	
	pub fn get(&mut self, offset: u64) -> &[u8] {
		// Get index if we have a entry for the offset
		let index = if let Some(index) = self.offset.get(&offset) { *index }
			else { panic!("Chunk is not available in cache") };
		
		// Update access-time and return entry
		self.update_timestamp(index);
		&self.entries.get(index).unwrap().data
	}
}
