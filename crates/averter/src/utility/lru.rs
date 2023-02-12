use std::collections::{HashMap, VecDeque};

const LRU_MAX_SIZE: usize = 100;

#[derive(Debug)]
pub struct LRU {
	entries: HashMap<String, String>,
	queue: VecDeque<String>,
}

impl LRU {
	pub fn new() -> Self {
		Self {
			entries: HashMap::with_capacity(LRU_MAX_SIZE),
			queue: VecDeque::with_capacity(LRU_MAX_SIZE),
		}
	}

	pub fn get(&mut self, key: String) -> Option<String> {
		if !self.entries.contains_key(&key) {
			return None;
		}

		if cfg!(debug_assertions) {
			println!("cache -> GET {}", key);
		}

		// Only update the front of the queue if the key is not already at the front
		match self.queue.front() {
			Some(front) => {
				if front != &key {
					self.queue.push_front(key.clone());
				}
			}
			None => {
				self.queue.push_front(key.clone());
			}
		}

		self.entries.get(&key).cloned()
	}

	pub fn insert(&mut self, key: String, value: String) {
		if cfg!(debug_assertions) {
			println!("cache -> SET {}", key);
		}

		self.entries.insert(key.to_owned(), value);
		self.queue.push_front(key.to_owned());
		println!("cache -> VAL {:#?}", self.queue);

		// Pops from the queue and removes the entry from the map
		// Only runs if the queue is larger than the max size
		if self.queue.len() > LRU_MAX_SIZE {
			if let Some(key) = self.queue.pop_back() {
				if cfg!(debug_assertions) {
					println!("cache -> POP {key}");
					println!("cache -> VAL {:#?}", self.queue);
				}

				self.entries.remove(&key);
			}
		}
	}
}
