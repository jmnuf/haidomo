use std::io::Write;
use std::time::Duration;

macro_rules! len_is_u8 {
    ($vec: expr,  $msg: expr) => {
        if $vec.len() > u8::MAX as usize {
            return Err($msg);
        }
    };
}

macro_rules! push_number_bytes {
    ($vec: expr, $number: expr) => {
        for b in $number.to_le_bytes() {
            $vec.push(b);
        }
    };
}
macro_rules! push_str_bytes {
    ($vec: expr, $str: expr) => {
        for b in $str.clone().into_bytes() {
            $vec.push(b);
        }
    };
}

const VERSION: u8 = 0b00000000;
const SIGNATURE: [u8; 4] = [b'b', b's', b's', 69];

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
            version: VERSION,
            name: name,
            splits: splits_names,
            attempts: vec![],
        }
    }

    pub fn add_attempt(&mut self, split_durations: Vec<Duration>) {
        let mut total_duration = Duration::ZERO;
        let mut split_times: Vec<f64> = Vec::new();
        for sd in split_durations.into_iter() {
            total_duration += sd;
            split_times.push(sd.as_secs_f64());
        }
        self.attempts.push(AttemptData {
            total_duration: total_duration,
            split_times: split_times,
        });
    }

    pub fn as_bytes(&self) -> Result<Vec<u8>, String> {
        let mut bytes = Vec::new();

	for b in &SIGNATURE {
	    bytes.push(*b);
	}

        bytes.push(self.version);

        len_is_u8!(
            self.name,
            format!(
                "Run name exceeds maximum. Got length {} but max is {}",
                self.name.len(),
                u8::MAX
            )
        );
        bytes.push(self.name.len() as u8);
        push_str_bytes!(bytes, self.name);

        // Add split related things, right now only the names
        let splits_count = self.splits.len();
        len_is_u8!(self.splits, format!("The amount of splits exceeds maximum. There are {splits_count} recorded splits but only a max of {} are allowed", u8::MAX));
        bytes.push(self.splits.len() as u8);
        for i in 0..splits_count {
            let split = &self.splits[i];
            let str_len = split.len();
            len_is_u8!(split, format!("Split {split} has a name that's too long. It has a length of {str_len} but it can only reach to be {}", u8::MAX));
            bytes.push(str_len as u8);
            push_str_bytes!(bytes, split);
        }

        // Add attempt durations and splits reached
        let attempts_count = self.attempts.len();
        len_is_u8!(self.attempts, format!("Too many attempts recorded. There are {attempts_count} recorded attempts but only a max of {} are allowed", u8::MAX));
        bytes.push(attempts_count as u8);
        for i in 0..attempts_count {
            let attempt = &self.attempts[i];
            // Total time
            let seconds = attempt.total_duration.as_secs();
            let nanos = attempt.total_duration.subsec_nanos();
            push_number_bytes!(bytes, seconds);
            push_number_bytes!(bytes, nanos);
            // Splits Used
            let splits_used = attempt.split_times.len();
            if splits_used > splits_count {
                let err_msg = format!("Attempt {i} has more splits times than the run holds! Max splits used per attempt is {splits_count} but attempt says it used {splits_used}!");
                return Err(err_msg);
            }
            bytes.push(splits_used as u8);
            for secs in attempt.split_times.iter() {
                push_number_bytes!(bytes, secs);
            }
        }

        return Ok(bytes);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn serialize_splits() {
	let expected_header: Vec<u8> = vec![
	    // File Signature
	    SIGNATURE[0],
	    SIGNATURE[1],
	    SIGNATURE[2],
	    SIGNATURE[3],
            // Version Number
            VERSION, // Run Name length
            0b00000100, // 4
            // UTF-8 Characters
            0b01110100, // 't'
            0b01100101, // 'e'
            0b01110011, // 's'
            0b01110100, // 't'
	];

	let expected_splits: Vec<u8> = vec![
            // Splits Count: 3
            0b00000011, // Split 1
            // Split Name Length
            0b00000010, // 2
            // UTF-8 Characters
            0b01010011, // 'S'
            0b00110001, // '1'
            // Split 2
            // Split Name Length
            0b00000010, // 2
            // UTF-8 Characters
            0b01010011, // 'S'
            0b00110010, // '2'
            // Split 3
            // Split Name Length
            0b00000010, // 2
            // UTF-8 Characters
            0b01010011, // 'S'
            0b00110011, // '3'
	];

	// TODO: Make it so I don't have to specify the bytes manually for the duration numbers to be able to test for multiple attempts
	let expected_attempts: Vec<u8> = vec![
            // Attempts Count
            0b00000001, // 1
            // Attempt 1 seconds duration: 9
            0b00001001, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
            0b00000000, // Attempt 1 fractional nanos duration: 700,000,000
            0b00000000, 0b00100111, 0b10111001, 0b00101001,
            // Attempt 1 splits used
            0b00000011, // 3
            // Attempt 1 Split 1 duration as f64: 3.21
            0b10101110, 0b01000111, 0b11100001, 0b01111010, 0b00010100, 0b10101110, 0b00001001,
            0b01000000, // Attempt 1 Split 2 duration as f64: 3.23
            0b11010111, 0b10100011, 0b01110000, 0b00111101, 0b00001010, 0b11010111, 0b00001001,
            0b01000000, // Attempt 1 Split 3 duration as f64: 3.26
            0b00010100, 0b10101110, 0b01000111, 0b11100001, 0b01111010, 0b00010100, 0b00001010,
            0b01000000,
	];

	// Generate the run data
	let mut rund = RunData::new(
            String::from("test"),
            vec!["S1".to_string(), "S2".to_string(), "S3".to_string()],
	);
	rund.add_attempt(vec![
            Duration::from_secs_f64(3.21),
            Duration::from_secs_f64(3.23),
            Duration::from_secs_f64(3.26),
	]);

	let result = rund
            .as_bytes()
            .expect("Should be able to transform test run data to bytes without problems");
	let mut res_idx = 0;
	let res_len = result.len();

	// Test Header section
	// TODO: Expand it to how the Attempts section is for easier understanding
	assert!(
            res_len > expected_header.len(),
            "Generated header section is too small! Expected {} but got {}",
            expected_header.len(),
            res_len - res_idx
	);
	let section = &result[res_idx..expected_header.len()];
	assert_eq!(
            &expected_header, section,
            "The generated header section (right) doesn't match with the expected header (left)!"
	);
	res_idx += expected_header.len();

	// Test Splits section
	// TODO> Expand it to how the Attempts section is for easier understanding
	assert!(
            res_len - res_idx > expected_splits.len(),
            "Generated splits section is too small! Expected {} but got {}",
            expected_splits.len(),
            res_len - res_idx
	);
	let section = &result[res_idx..(res_idx + expected_splits.len())];
	assert_eq!(
            &expected_splits, section,
            "The generated splits section (right) doesn't match with the expected splits (left)!"
	);
	res_idx += expected_splits.len();

	// Test Attempts section
	// God save me I spent too much time here because I misstyped 2 bits in the expected data :'v
	assert!(
            res_len - res_idx >= expected_attempts.len(),
            "Generated attempts section is too small! Expected {} but got {}",
            expected_attempts.len(),
            res_len - res_idx
	);

	let got = u8::from_le(*result.iter().skip(res_idx).take(1).collect::<Vec<&u8>>()[0]);
	let exp = u8::from_le(expected_attempts[0]);
	assert_eq!(
            exp, got,
            "Attempts count is expected to be {exp} ({exp:#010b}) but got {got} ({got:#010b})"
	);

	let mut offset = 1usize;
	let got = u64::from_le_bytes({
            let v: Vec<u8> = result
		.iter()
		.skip(res_idx + offset)
		.take(8)
		.map(|x| *x)
		.collect();
            let v: [u8; 8] = v.try_into().expect("Should be able to create a [u8; 8]");
            v
	});
	let exp = u64::from_le_bytes({
            let v: Vec<u8> = expected_attempts
		.iter()
		.skip(offset)
		.take(8)
		.map(|x| *x)
		.collect();
            let v: [u8; 8] = v.try_into().expect("Should be able to create a [u8; 8]");
            v
	});
	assert_eq!(exp, got, "Attempt total duration seconds expected to be {exp} ({exp:#066b}) but got {got} ({got:#066b})");

	offset += 8;
	let got = u32::from_le_bytes({
            let v: Vec<u8> = result
		.iter()
		.skip(res_idx + offset)
		.take(4)
		.map(|x| *x)
		.collect();
            let v: [u8; 4] = v.try_into().expect("Should be able to create a [u8;4]");
            v
	});
	let exp = u32::from_le_bytes({
            let v: Vec<u8> = expected_attempts
		.iter()
		.skip(offset)
		.take(4)
		.map(|x| *x)
		.collect();
            let v: [u8; 4] = v.try_into().expect("Should be able to create a [u8; 4]");
            v
	});
	assert_eq!(exp, got, "Attempt total duration nanos subsec expected to be {exp} ({exp:#034b}) but got {got} ({got:#034b})");

	offset += 4;
	let got = u8::from_le({
            let v: Vec<u8> = result
		.iter()
		.skip(res_idx + offset)
		.take(1)
		.map(|x| *x)
		.collect();
            let v: [u8; 1] = v.try_into().expect("Should be able to create a [u8; 1]");
            v[0]
	});
	let exp = u8::from_le({
            let v: Vec<u8> = expected_attempts
		.iter()
		.skip(offset)
		.take(1)
		.map(|x| *x)
		.collect();
            let v: [u8; 1] = v.try_into().expect("Should be able to create a [u8; 1]");
            v[0]
	});
	assert_eq!(exp, got, "Expected attempt's splits used area to be equal to: {exp} ({exp:#010b}) but got {got} ({got:#010b})");

	let section = &result[res_idx..(res_idx + expected_attempts.len())];
	assert_eq!(
            &expected_attempts, section,
            "The generated attempts section (right) doesn't match with the expected attempts (left)!"
	);
	res_idx += expected_attempts.len();

	assert_eq!(
            res_idx,
            result.len(),
            "Generated extra bytes: {:?}",
            result.iter().skip(res_idx).collect::<Vec<&u8>>()
	);
    }

    #[test]
    fn write() {
	let contents: Vec<u8> = vec![
            // Version Number
            VERSION, // Run Name length
            0b00000100, // 4
            // UTF-8 Characters
            0b01110100, // 't'
            0b01100101, // 'e'
            0b01110011, // 's'
            0b01110100, // 't'

            // Splits Count: 3
            0b00000011, // Split 1
            // Split Name Length
            0b00000010, // 2
            // UTF-8 Characters
            0b01010011, // 'S'
            0b00110001, // '1'
            // Split 2
            // Split Name Length
            0b00000010, // 2
            // UTF-8 Characters
            0b01010011, // 'S'
            0b00110010, // '2'
            // Split 3
            // Split Name Length
            0b00000010, // 2
            // UTF-8 Characters
            0b01010011, // 'S'
            0b00110011, // '3'
	    
            // Attempts Count
            0b00000001, // 1
            // Attempt 1 seconds duration: 9
            0b00001001, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
            0b00000000, // Attempt 1 fractional nanos duration: 700,000,000
            0b00000000, 0b00100111, 0b10111001, 0b00101001,
            // Attempt 1 splits used
            0b00000011, // 3
            // Attempt 1 Split 1 duration as f64: 3.21
            0b10101110, 0b01000111, 0b11100001, 0b01111010, 0b00010100, 0b10101110, 0b00001001,
            0b01000000, // Attempt 1 Split 2 duration as f64: 3.23
            0b11010111, 0b10100011, 0b01110000, 0b00111101, 0b00001010, 0b11010111, 0b00001001,
            0b01000000, // Attempt 1 Split 3 duration as f64: 3.26
            0b00010100, 0b10101110, 0b01000111, 0b11100001, 0b01111010, 0b00010100, 0b00001010,
            0b01000000,
	];

	let mut f = std::fs::File::create("foo.bss").unwrap();
	f.write_all(&contents).expect("Expected to manage to write all the buffer onto the file");
	f.flush().expect("Expected to be able to flush file after write");
    }

    // TODO: Feature not implemented
    // #[test]
    // fn read_serialized_data() {
    // }
}
