use std::time::Duration;
use std::io::Write;

macro_rules! len_is_u8 {
    ($vec: expr,  $msg: expr) => {
	if $vec.len() > u8::MAX as usize {
	    return Err($msg);
	}
    }
}

const VERSION: u8 = 0;

struct RunData {
    version: u8,
    name: String,
    splits: Vec<String>,
    attempts: Vec<AttemptData>,
}
struct AttemptData {
    total_duration: Duration,
    split_times: Vec<f64>,
}


impl RunData {
    pub fn new(name: String, splits_names: Vec<String>) -> Self {
	Self {
	    version:  VERSION,
	    name: name,
	    splits: splits_names,
	    attempts: vec![],
	}
    }
    
    pub fn as_bytes(&self) -> Result<Vec<u8>, String> {
	let bytes = Vec::new();
	bytes.push(self.version);
	for b in self.name.as_bytes() {
	    bytes.push(b);
	}
	let splits_count = self.splits.len();
	len_is_u8!(self.splits, format!("The amount of splits exceeds maximum. There are {splits_count} recorded splits but only a max of {} are allowed", u8::MAX));
	bytes.push(self.splits.len() as u8);
	for i in splits_count {
	    let split = splits[i];
	    let str_len = split.len();
	    len_is_u8!(split, format!("Split {split} has a name that's too long. It has a length of {str_len} but it can only reach to be {}", u8::MAX));
	    for b in (str_len as u16).to_le_bytes() {
		bytes.push(b);
	    }
	}

	let attempts_count = self.attempts.len();
	len_is_u8!(self.attempts, format!("Too many attempts recorded. There are {attempts_count} recorded attempts but only a max of {} are allowed", u8::MAX));
	for i in attempts_count {
	    // TODO: Serialize attempts into bytes
	}
    }
}
