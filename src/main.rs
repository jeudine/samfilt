extern crate getopts;
use getopts::Options;
use std::env::args;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
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

	let param = FilterParam {
		supplementary: supplementary,
		greater_len: greater_len,
		smaller_len: smaller_len,
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
	filter(&input, &mut *output, param);
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

#[derive(Clone, Copy)]
struct FilterParam {
	supplementary: Mode,
	greater_len: u32,
	smaller_len: u32,
}

struct ReadSpec {
	has_supplementary: bool,
	len: u32,
}

fn filter(input: &File, output: &mut dyn io::Write, param: FilterParam) {
	let reader = BufReader::new(input);
	let mut name_buf = "".to_string();
	let mut line_buf: Vec<String> = Vec::new();
	let mut spec = ReadSpec {
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
			if name_buf == name {
				if (flag & 2048) != 0 {
					spec.has_supplementary = true;
				}
			} else {
				if name_buf != "" {
					write_filter(&line_buf, &spec, &param, output);
				}
				name_buf = name.to_string();
				spec.has_supplementary = false;
				spec.len = field[9].len() as u32;
				//println!("{:?}: {:?}", name_buf, spec.len);
				line_buf.clear();
			}
			line_buf.push(line.to_string());
		}
	}
	if name_buf != "" {
		write_filter(&line_buf, &spec, &param, output);
	}
}

#[inline]
fn write_filter(
	lines: &Vec<String>,
	spec: &ReadSpec,
	param: &FilterParam,
	output: &mut dyn io::Write,
) {
	if param.supplementary == Mode::Sel && !spec.has_supplementary {
		return;
	}

	if param.supplementary == Mode::Del && spec.has_supplementary {
		return;
	}

	if param.greater_len >= spec.len {
		return;
	}

	if param.smaller_len <= spec.len {
		return;
	}

	write_sam(lines, output);
}

#[inline]
fn write_sam(lines: &Vec<String>, output: &mut dyn io::Write) {
	for line in lines {
		if let Err(err) = writeln!(output, "{}", line) {
			eprintln!("[ERROR] write {}", err);
			process::exit(1);
		}
	}
}
