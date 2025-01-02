/// Common functionality for all integration tests
/// Note: If you encounter `error[E0463]: can't find crate for 'test'`, rememmber to add
/// `harness = false` to the [[test]] section of cargo.toml
use hermit::{print, println};

//use std::borrow::Cow;
//use std::fmt;

//From libtest types.rs-----------------------------------------------------
/*
/// Type of the test according to the [rust book](https://doc.rust-lang.org/cargo/guide/tests.html)
/// conventions.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TestType {
	/// Unit-tests are expected to be in the `src` folder of the crate.
	UnitTest,
	/// Integration-style tests are expected to be in the `tests` folder of the crate.
	IntegrationTest,
	/// Doctests are created by the `librustdoc` manually, so it's a different type of test.
	DocTest,
	/// Tests for the sources that don't follow the project layout convention
	/// (e.g. tests in raw `main.rs` compiled by calling `rustc --test` directly).
	Unknown,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum NamePadding {
	PadNone,
	PadOnRight,
}

// The name of a test. By convention this follows the rules for rust
// paths; i.e., it should be a series of identifiers separated by double
// colons. This way if some test runner wants to arrange the tests
// hierarchically it may.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TestName {
	StaticTestName(&'static str),
	DynTestName(String),
	AlignedTestName(Cow<'static, str>, NamePadding),
}

impl TestName {
	pub fn as_slice(&self) -> &str {
		match *self {
			StaticTestName(s) => s,
			DynTestName(ref s) => s,
			AlignedTestName(ref s, _) => &*s,
		}
	}

	pub fn padding(&self) -> NamePadding {
		match self {
			&AlignedTestName(_, p) => p,
			_ => PadNone,
		}
	}

	pub fn with_padding(&self, padding: NamePadding) -> TestName {
		let name = match *self {
			TestName::StaticTestName(name) => Cow::Borrowed(name),
			TestName::DynTestName(ref name) => Cow::Owned(name.clone()),
			TestName::AlignedTestName(ref name, _) => name.clone(),
		};

		TestName::AlignedTestName(name, padding)
	}
}
impl fmt::Display for TestName {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Display::fmt(self.as_slice(), f)
	}
}

/// Represents a benchmark function.
pub trait TDynBenchFn: Send {
	fn run(&self, harness: &mut Bencher);
}

// A function that runs a test. If the function returns successfully,
// the test succeeds; if the function panics then the test fails. We
// may need to come up with a more clever definition of test in order
// to support isolation of tests into threads.
pub enum TestFn {
	StaticTestFn(fn()),
	StaticBenchFn(fn(&mut Bencher)),
	DynTestFn(Box<dyn FnOnce() + Send>),
	DynBenchFn(Box<dyn TDynBenchFn + 'static>),
}

impl TestFn {
	pub fn padding(&self) -> NamePadding {
		match *self {
			StaticTestFn(..) => PadNone,
			StaticBenchFn(..) => PadOnRight,
			DynTestFn(..) => PadNone,
			DynBenchFn(..) => PadOnRight,
		}
	}
}

impl fmt::Debug for TestFn {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(match *self {
			StaticTestFn(..) => "StaticTestFn(..)",
			StaticBenchFn(..) => "StaticBenchFn(..)",
			DynTestFn(..) => "DynTestFn(..)",
			DynBenchFn(..) => "DynBenchFn(..)",
		})
	}
}

// The definition of a single test. A test runner will run a list of
// these.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TestDesc {
	pub name: TestName,
	pub ignore: bool,
	pub should_panic: options::ShouldPanic,
	pub allow_fail: bool,
	pub test_type: TestType,
}

impl TestDesc {
	pub fn padded_name(&self, column_count: usize, align: NamePadding) -> String {
		let mut name = String::from(self.name.as_slice());
		let fill = column_count.saturating_sub(name.len());
		let pad = " ".repeat(fill);
		match align {
			PadNone => name,
			PadOnRight => {
				name.push_str(&pad);
				name
			}
		}
	}
}

#[derive(Debug)]
pub struct TestDescAndFn {
	pub desc: TestDesc,
	pub testfn: TestFn,
}
*/
// End from libtest ------------------------------------------------------------
/*
extern crate test;
use self::test::TestFn::StaticTestFn;
use test::TestDescAndFn;


pub fn test_runner(tests: &[&TestDescAndFn]) {
	println!("Running {} tests", tests.len());
	for test in tests {
		let TestDescAndFn { desc, testfn } = test;
		println!("Running {}", desc.name);
		match testfn {
			StaticTestFn(f) => f(),
			_ => panic!("hermit currently only support Static test functions"),
		}
	}
	exit(false);
}*/

/// For test_case (without `TestDesc`)
pub trait Testable {
	fn run(&self);
}

impl<T> Testable for T
where
	T: Fn(),
{
	fn run(&self) {
		print!("{}...\t", core::any::type_name::<T>());
		self();
		println!("[ok]");
	}
}

#[allow(dead_code)]
pub fn test_case_runner(tests: &[&dyn Testable]) {
	println!("Running {} tests", tests.len());
	for test in tests {
		test.run();
	}
	exit(false);
}

pub fn exit(failure: bool) -> ! {
	match failure {
		true => hermit::syscalls::sys_exit(1),
		false => hermit::syscalls::sys_exit(0),
	}
}

/// defines runtime_entry and passes arguments as Rust String to main method with signature:
/// `fn main(args: Vec<String>) -> Result<(), ()>;`
#[macro_export]
macro_rules! runtime_entry_with_args {
	() => {
		// ToDo: Maybe we could add a hard limit on the length of `s` to make this slightly safer?
		unsafe fn parse_str(s: *const u8) -> Result<alloc::string::String, ()> {
			use alloc::string::String;
			use alloc::vec::Vec;

			let mut vec: Vec<u8> = Vec::new();
			let mut off = s;
			while unsafe { *off } != 0 {
				vec.push(unsafe { *off });
				off = unsafe { off.offset(1) };
			}
			let str = String::from_utf8(vec);
			match str {
				Ok(s) => Ok(s),
				Err(_) => Err(()), //Convert error here since we might want to add another error type later
			}
		}

		#[no_mangle]
		extern "C" fn runtime_entry(
			argc: i32,
			argv: *const *const u8,
			_env: *const *const u8,
		) -> ! {
			use alloc::string::String;
			use alloc::vec::Vec;

			let mut str_vec: Vec<String> = Vec::new();
			let mut off = argv;
			for i in 0..argc {
				let s = unsafe { parse_str(*off) };
				unsafe {
					off = off.offset(1);
				}
				match s {
					Ok(s) => str_vec.push(s),
					Err(_) => println!(
						"Warning: Application argument {} is not valid utf-8 - Dropping it",
						i
					),
				}
			}

			let res = main(str_vec);
			match res {
				Ok(_) => common::exit(false),
				Err(_) => common::exit(true),
			}
		}
	};
}
