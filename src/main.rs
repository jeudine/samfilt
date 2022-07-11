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

	let mut param: FilterParam = Default::default();

	param.supplementary = match matches.opt_get_default("supplementary", Mode::No) {
		Ok(m) => m,
		Err(err) => {
			eprintln!("[ERROR] {}: 'supplementary'", err);
			process::exit(1);
		}
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

#[derive(Clone, Copy, Default)]
enum Mode {
	#[default]
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

#[derive(Clone, Copy, Default)]
struct FilterParam {
	supplementary: Mode,
}

struct ReadSpec {
	has_supplementary: bool,
}

fn filter(input: &File, output: &mut dyn io::Write, param: FilterParam) {
	let reader = BufReader::new(input);
	let mut name_buf = "".to_string();
	let mut line_buf: Vec<String> = Vec::new();
	let mut spec = ReadSpec {
		has_supplementary: false,
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
			if name_buf.eq(name) {
				if (flag & 2048) != 0 {
					spec.has_supplementary = true;
				}
			} else {
				if !name_buf.eq("") {
					write_filter(&line_buf, &spec, &param, output);
				}
				name_buf = name.to_string();
				spec.has_supplementary = false;
				line_buf.clear();
			}
			line_buf.push(line.to_string());
		}
	}
	if !name_buf.eq("") {
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
	match param.supplementary {
		Mode::No => write_sam(lines, output),
		Mode::Sel => {
			if spec.has_supplementary {
				write_sam(lines, output);
			}
		}
		Mode::Del => {
			if !spec.has_supplementary {
				write_sam(lines, output);
			}
		}
	};
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
