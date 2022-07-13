extern crate getopts;
use getopts::Options;
use std::env::args;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process;
use std::str::FromStr;

fn main() {
	let args: Vec<_> = args().collect();
	let program = args[0].clone();

	let mut opts = Options::new();
	opts.optflag("h", "help", "print this help menu");
	opts.optopt(
		"",
		"supplementary",
		"reads with supplementary alignments",
		"sel|del",
	);
	opts.optopt(
		"",
		"greater_len",
		"reads with a greater length than UINT [0]",
		"UINT",
	);
	opts.optopt(
		"",
		"smaller_len",
		"reads with a smaller length than UINT [4294967295]",
		"UINT",
	);
	opts.optopt(
		"",
		"qname_input",
		"alignment records with QNAME being equal to one of the lines in FILE",
		"FILE",
	);

	opts.optopt(
		"",
		"qname_output",
		"output the QNAME field of all the records in the filtered SAM file (one per line)",
		"FILE",
	);

	opts.optopt("o", "", "output to FILE [stdout]", "FILE");

	let matches = match opts.parse(&args[1..]) {
		Ok(m) => m,
		Err(f) => {
			eprintln!("[ERROR] {}", f.to_string());
			print_usage(&program, opts);
			process::exit(1);
		}
	};

	if matches.opt_present("h") {
		print_usage(&program, opts);
		return;
	}

	let supplementary = match matches.opt_get_default("supplementary", Mode::No) {
		Ok(m) => m,
		Err(err) => {
			eprintln!("[ERROR] {}: 'supplementary'", err);
			process::exit(1);
		}
	};

	let greater_len = match matches.opt_get_default("greater_len", 0) {
		Ok(m) => m,
		Err(err) => {
			eprintln!("[ERROR] {}: 'greater_len'", err);
			process::exit(1);
		}
	};

	let smaller_len = match matches.opt_get_default("smaller_len", u32::MAX) {
		Ok(m) => m,
		Err(err) => {
			eprintln!("[ERROR] {}: 'smaller_len'", err);
			process::exit(1);
		}
	};

	let qname_input = match matches.opt_str("qname_input") {
		Some(ref p) => {
			let path = Path::new(p);
			let file = match File::open(&path) {
				Ok(file) => file,
				Err(err) => {
					eprintln!("[ERROR] open: {} {}", path.display(), err);
					process::exit(1);
				}
			};
			let reader = BufReader::new(&file);
			Some(
				reader
					.lines()
					.map(|line| match line {
						Ok(l) => l,
						Err(err) => {
							eprintln!("[ERROR] line: {}", err);
							process::exit(1);
						}
					})
					.collect(),
			)
		}
		None => None,
	};

	let qname_output = match matches.opt_str("qname_output") {
		Some(ref p) => {
			let path = Path::new(p);
			match File::create(&path) {
				Ok(file) => Some(file),
				Err(err) => {
					eprintln!("[ERROR] create: {} {}", path.display(), err);
					process::exit(1);
				}
			}
		}
		None => None,
	};

	let mut param = FilterParam {
		supplementary: supplementary,
		greater_len: greater_len,
		smaller_len: smaller_len,
		qname_input: qname_input,
		qname_output: qname_output,
	};

	if matches.free.len() != 1 {
		print_usage(&program, opts);
		process::exit(1);
	}

	let path = Path::new(&matches.free[0]);
	let input = match File::open(&path) {
		Ok(file) => file,
		Err(err) => {
			eprintln!("[ERROR] open: {} {}", path.display(), err);
			process::exit(1);
		}
	};

	let mut output: Box<dyn io::Write> = match matches.opt_str("o") {
		Some(ref o) => {
			let path = Path::new(o);
			Box::new(match File::create(&path) {
				Ok(file) => file,
				Err(err) => {
					eprintln!("[ERROR] open: {} {}", path.display(), err);
					process::exit(1);
				}
			})
		}
		None => Box::new(io::stdout()),
	};
	filter(&input, &mut *output, &mut param);
}

fn print_usage(program: &str, opts: Options) {
	let brief = format!("Usage: {} [options] <SAM file>", program);
	print!("{}", opts.usage(&brief));
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
	No,
	Sel,
	Del,
}

impl FromStr for Mode {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s == "sel" {
			Ok(Mode::Sel)
		} else if s == "del" {
			Ok(Mode::Del)
		} else {
			Err("unrecognized mode".to_string())
		}
	}
}

struct FilterParam {
	supplementary: Mode,
	greater_len: u32,
	smaller_len: u32,
	qname_input: Option<Vec<String>>,
	qname_output: Option<File>,
}

struct Read {
	qname: String,
	has_supplementary: bool,
	len: u32,
}

fn filter(input: &File, output: &mut dyn io::Write, mut param: &mut FilterParam) {
	let reader = BufReader::new(input);
	let mut line_buf: Vec<String> = Vec::new();
	let mut read = Read {
		qname: "".to_string(),
		has_supplementary: false,
		len: 0,
	};
	for line in reader.lines() {
		let line = match line {
			Ok(l) => l,
			Err(err) => {
				eprintln!("[ERROR] line: {}", err);
				process::exit(1);
			}
		};
		let field: Vec<_> = line.split('\t').collect();
		let name = field[0];
		if name.chars().next().unwrap() == '@' {
			if let Err(err) = writeln!(output, "{}", line) {
				eprintln!("[ERROR] write {}", err);
				process::exit(1);
			}
		} else {
			let flag: u16 = field[1].parse().unwrap();
			if read.qname == name {
				if (flag & 2048) != 0 {
					read.has_supplementary = true;
				}
			} else {
				if read.qname != "" {
					write_filter(&line_buf, &read, &mut param, output);
				}
				read.qname = name.to_string();
				read.has_supplementary = false;
				read.len = field[9].len() as u32;
				line_buf.clear();
			}
			line_buf.push(line.to_string());
		}
	}
	if read.qname != "" {
		write_filter(&line_buf, &read, &mut param, output);
	}
}

#[inline]
fn write_filter(
	lines: &Vec<String>,
	read: &Read,
	param: &mut FilterParam,
	output: &mut dyn io::Write,
) {
	//println!("{:?}: {:?}", read.qname, read.len);
	if let Some(ref qname_input) = param.qname_input {
		let mut is_present = false;
		for name in qname_input {
			if read.qname == *name {
				is_present = true;
				break;
			}
		}
		if !is_present {
			return;
		}
	}

	if param.supplementary == Mode::Sel && !read.has_supplementary {
		return;
	}

	if param.supplementary == Mode::Del && read.has_supplementary {
		return;
	}

	if param.greater_len >= read.len {
		return;
	}

	if param.smaller_len <= read.len {
		return;
	}

	write_sam(lines, output, &mut param.qname_output, &read.qname);
}

#[inline]
fn write_sam(
	lines: &Vec<String>,
	output: &mut dyn io::Write,
	qname_output: &mut Option<File>,
	qname: &str,
) {
	for line in lines {
		if let Err(err) = writeln!(output, "{}", line) {
			eprintln!("[ERROR] write {}", err);
			process::exit(1);
		}
	}
	if let Some(file) = qname_output {
		if let Err(err) = writeln!(file, "{}", qname) {
			eprintln!("[ERROR] write {}", err);
			process::exit(1);
		}
	}
}
