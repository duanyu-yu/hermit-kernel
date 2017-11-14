// Copyright (c) 2017 Stefan Lankes, RWTH Aachen University
//                    Colin Finck, RWTH Aachen University
//
// MIT License
//
// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the
// "Software"), to deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to
// permit persons to whom the Software is furnished to do so, subject to
// the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
// LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use arch::x86_64::irq;
use arch::x86_64::percore::*;
use arch::x86_64::pit;
use core::{fmt, ptr, slice, str};
use logging::*;
use raw_cpuid::*;
use tasks::*;
use x86::shared::control_regs::*;
use x86::shared::msr::*;
use x86::shared::time::*;


extern "C" {
	#[link_section = ".percore"]
	static __core_id: u32;

	static cmdline: *const u8;
	static cmdsize: usize;
	static current_boot_id: i32;
	static mut Lpatch0: u8;
	static mut Lpatch1: u8;
	static mut Lpatch2: u8;
	static percore_start: u8;
	static percore_end0: u8;
}

#[link_section = ".percore"]
#[no_mangle]
/// Value returned by RDTSC/RDTSCP last time we checked.
pub static mut last_rdtsc: u64 = 0;

#[link_section = ".percore"]
#[no_mangle]
/// Counted ticks of a timer with the constant frequency specified in TIMER_FREQUENCY.
pub static mut timer_ticks: u64 = 0;

/// Timer frequency in Hz for the timer_ticks.
pub const TIMER_FREQUENCY: u64 = 100;

const EFER_NXE: u64 = 1 << 11;
const IA32_MISC_ENABLE_ENHANCED_SPEEDSTEP: u64 = 1 << 16;
const IA32_MISC_ENABLE_SPEEDSTEP_LOCK: u64 = 1 << 20;
const IA32_MISC_ENABLE_TURBO_DISABLE: u64 = 1 << 38;


static mut CPU_FREQUENCY: CpuFrequency = CpuFrequency::new();
static mut CPU_SPEEDSTEP: CpuSpeedStep = CpuSpeedStep::new();
static mut PHYSICAL_ADDRESS_BITS: u8 = 0;
static mut LINEAR_ADDRESS_BITS: u8 = 0;
static mut MEASUREMENT_TIMER_TICKS: u64 = 0;
static mut SUPPORTS_1GIB_PAGES: bool = false;
static mut SUPPORTS_AVX: bool = false;
static mut SUPPORTS_FSGSBASE: bool = false;
static mut SUPPORTS_XSAVE: bool = false;
static mut TIMESTAMP_FUNCTION: unsafe fn() -> u64 = get_timestamp_rdtsc;


#[repr(C, align(16))]
pub struct XSaveLegacyRegion {
	pub fpu_control_word: u16,
	pub fpu_status_word: u16,
	pub fpu_tag_word: u16,
	pub fpu_opcode: u16,
	pub fpu_instruction_pointer: u32,
	pub fpu_instruction_pointer_high_or_cs: u32,
	pub fpu_data_pointer: u32,
	pub fpu_data_pointer_high_or_ds: u32,
	pub mxcsr: u32,
	pub mxcsr_mask: u32,
	pub st_space: [u8; 8*16],
	pub xmm_space: [u8; 16*16],
	pub padding: [u8; 96],
}

#[repr(C)]
pub struct XSaveHeader {
	pub xstate_bv: u64,
	pub xcomp_bv: u64,
	pub reserved: [u64; 6],
}

#[repr(C)]
pub struct XSaveAVXState {
	pub ymmh_space: [u8; 16*16],
}

/// XSave Area for AMD Lightweight Profiling.
/// Refer to AMD Lightweight Profiling Specification (Publication No. 43724), Figure 7-1.
#[repr(C)]
pub struct XSaveLWPState {
	pub lwpcb_address: u64,
	pub flags: u32,
	pub buffer_head_offset: u32,
	pub buffer_base: u64,
	pub buffer_size: u32,
	pub filters: u32,
	pub saved_event_record: [u64; 4],
	pub event_counter: [u32; 16],
}

#[repr(C)]
pub struct XSaveBndregs {
	pub bound_registers: [u8; 4*16],
}

#[repr(C)]
pub struct XSaveBndcsr {
	pub bndcfgu_register: u64,
	pub bndstatus_register: u64,
}

#[repr(C, align(64))]
pub struct XSaveArea {
	pub legacy_region: XSaveLegacyRegion,
	pub header: XSaveHeader,
	pub avx_state: XSaveAVXState,
	pub lwp_state: XSaveLWPState,
	pub bndregs: XSaveBndregs,
	pub bndcsr: XSaveBndcsr,
}


enum CpuFrequencySources {
	Invalid,
	CommandLine,
	CpuIdFrequencyInfo,
	CpuIdBrandString,
	Measurement,
}

impl fmt::Display for CpuFrequencySources {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			&CpuFrequencySources::CommandLine => write!(f, "Command Line"),
			&CpuFrequencySources::CpuIdFrequencyInfo => write!(f, "CPUID Frequency Info"),
			&CpuFrequencySources::CpuIdBrandString => write!(f, "CPUID Brand String"),
			&CpuFrequencySources::Measurement => write!(f, "Measurement"),
			_ => panic!("Attempted to print an invalid CPU Frequency Source"),
		}
	}
}


struct CpuFrequency {
	mhz: u16,
	source: CpuFrequencySources,
}

impl CpuFrequency {
	const fn new() -> Self {
		CpuFrequency { mhz: 0, source: CpuFrequencySources::Invalid }
	}

	unsafe fn detect_from_cmdline(&mut self) -> bool {
		if cmdsize > 0 {
			let slice = slice::from_raw_parts(cmdline, cmdsize);
			let cmdline_str = str::from_utf8_unchecked(slice);

			let freq_find = cmdline_str.find("-freq");
			if freq_find.is_some() {
				let cmdline_freq_str = cmdline_str.split_at(freq_find.unwrap() + "-freq".len()).1;
				let mhz_str = cmdline_freq_str.split(' ').next().expect("Invalid -freq command line");

				self.mhz = mhz_str.parse().expect("Could not parse -freq command line as number");
				self.source = CpuFrequencySources::CommandLine;
				true
			} else {
				false
			}
		} else {
			false
		}
	}

	unsafe fn detect_from_cpuid_frequency_info(&mut self, cpuid: &CpuId) -> bool {
		if let Some(info) = cpuid.get_processor_frequency_info() {
			self.mhz = info.processor_base_frequency();
			self.source = CpuFrequencySources::CpuIdFrequencyInfo;
			true
		} else {
			false
		}
	}

	unsafe fn detect_from_cpuid_brand_string(&mut self, cpuid: &CpuId) -> bool {
		let extended_function_info = cpuid.get_extended_function_info().expect("CPUID Extended Function Info not available!");
		let brand_string = extended_function_info.processor_brand_string().expect("CPUID Brand String not available!");

		let ghz_find = brand_string.find("GHz");
		if ghz_find.is_some() {
			let index = ghz_find.unwrap() - 4;
			let thousand_char = brand_string.chars().nth(index).unwrap();
			let decimal_char = brand_string.chars().nth(index + 1).unwrap();
			let hundred_char = brand_string.chars().nth(index + 2).unwrap();
			let ten_char = brand_string.chars().nth(index + 3).unwrap();

			if let (Some(thousand), '.', Some(hundred), Some(ten)) = (thousand_char.to_digit(10), decimal_char, hundred_char.to_digit(10), ten_char.to_digit(10)) {
				self.mhz = (thousand * 1000 + hundred * 100 + ten * 10) as u16;
				self.source = CpuFrequencySources::CpuIdBrandString;
				true
			} else {
				false
			}
		} else {
			false
		}
	}

	fn measure_frequency_timer_handler(state_ref: &irq::state) {
		unsafe { MEASUREMENT_TIMER_TICKS += 1; }
	}

	fn measure_frequency(&mut self) -> bool {
		// Measure the CPU frequency by counting 3 ticks of a 100Hz timer.
		let tick_count = 3;
		let measurement_frequency = 100;

		// Use the Programmable Interval Timer (PIT) for this measurement, which is the only
		// system timer with a known constant frequency.
		irq::set_handler(pit::PIT_INTERRUPT_NUMBER, Self::measure_frequency_timer_handler);
		pit::init(measurement_frequency);

		// Determine the current timer tick.
		// We are probably loading this value in the middle of a time slice.
		let first_tick = unsafe { MEASUREMENT_TIMER_TICKS };

		// Wait until the tick count changes.
		// As soon as it has done, we are at the start of a new time slice.
		let start_tick = loop {
			let tick = unsafe { MEASUREMENT_TIMER_TICKS };
			if tick != first_tick {
				break tick;
			}

			pause();
		};

		// Count the number of CPU cycles during 3 timer ticks.
		let start = unsafe { TIMESTAMP_FUNCTION() };

		loop {
			let tick = unsafe { MEASUREMENT_TIMER_TICKS };
			if tick - start_tick >= tick_count {
				break;
			}

			pause();
		}

		let end = unsafe { TIMESTAMP_FUNCTION() };

		// Deinitialize the PIT again.
		// Now we can calculate our CPU frequency and implement a constant frequency tick counter
		// using RDTSC timestamps.
		pit::deinit();

		// Calculate the CPU frequency out of this measurement.
		let cycle_count = abs_diff(start, end);
		self.mhz = (measurement_frequency * cycle_count / (1_000_000 * tick_count)) as u16;
		self.source = CpuFrequencySources::Measurement;
		true
	}

	unsafe fn detect(&mut self) {
		let cpuid = CpuId::new();
		self.detect_from_cmdline()
			|| self.detect_from_cpuid_frequency_info(&cpuid)
			|| self.detect_from_cpuid_brand_string(&cpuid)
			|| self.measure_frequency();
	}

	fn get(&self) -> u16 {
		self.mhz
	}
}

impl fmt::Display for CpuFrequency {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{} MHz (from {})", self.mhz, self.source)
	}
}


struct CpuFeaturePrinter {
	feature_info: FeatureInfo,
	extended_feature_info: ExtendedFeatures,
	extended_function_info: ExtendedFunctionInfo,
}

impl CpuFeaturePrinter {
	fn new(cpuid: &CpuId) -> Self {
		CpuFeaturePrinter {
			feature_info: cpuid.get_feature_info().expect("CPUID Feature Info not available!"),
			extended_feature_info: cpuid.get_extended_feature_info().expect("CPUID Extended Feature Info not available!"),
			extended_function_info: cpuid.get_extended_function_info().expect("CPUID Extended Function Info not available!"),
		}
	}
}

impl fmt::Display for CpuFeaturePrinter {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if self.feature_info.has_mmx() { write!(f, "MMX ")?; }
		if self.feature_info.has_sse() { write!(f, "SSE ")?; }
		if self.feature_info.has_sse2() { write!(f, "SSE2 ")?; }
		if self.feature_info.has_sse3() { write!(f, "SSE3 ")?; }
		if self.feature_info.has_ssse3() { write!(f, "SSSE3 ")?; }
		if self.feature_info.has_sse41() { write!(f, "SSE4.1 ")?; }
		if self.feature_info.has_sse42() { write!(f, "SSE4.2 ")?; }
		if self.feature_info.has_avx() { write!(f, "AVX ")?; }
		if self.extended_feature_info.has_avx2() { write!(f, "AVX2 ")?; }
		if self.feature_info.has_eist() { write!(f, "EIST ")?; }
		if self.feature_info.has_aesni() { write!(f, "AESNI ")?; }
		if self.feature_info.has_rdrand() { write!(f, "RDRAND ")?; }
		if self.feature_info.has_fma() { write!(f, "FMA ")?; }
		if self.feature_info.has_movbe() { write!(f, "MOVBE ")?; }
		if self.feature_info.has_x2apic() { write!(f, "X2APIC ")?; }
		if self.feature_info.has_mce() { write!(f, "MCE ")?; }
		if self.feature_info.has_fxsave_fxstor() { write!(f, "FXSR ")?; }
		if self.feature_info.has_xsave() { write!(f, "XSAVE ")?; }
		if self.feature_info.has_vmx() { write!(f, "VMX ")?; }
		if self.extended_function_info.has_rdtscp() { write!(f, "RDTSCP ")?; }
		if self.feature_info.has_monitor_mwait() { write!(f, "MWAIT ")?; }
		if self.feature_info.has_clflush() { write!(f, "CLFLUSH ")?; }
		if self.extended_feature_info.has_bmi1() { write!(f, "BMI1 ")?; }
		if self.extended_feature_info.has_bmi2() { write!(f, "BMI2 ")?; }
		if self.extended_feature_info.has_fsgsbase() { write!(f, "FSGSBASE ")?; }
		if self.feature_info.has_dca() { write!(f, "DCA ")?; }
		if self.extended_feature_info.has_rtm() { write!(f, "RTM ")?; }
		if self.extended_feature_info.has_hle() { write!(f, "HLE ")?; }
		if self.extended_feature_info.has_qm() { write!(f, "CQM ")?; }
		if self.extended_feature_info.has_mpx() { write!(f, "MPX ")?; }
		Ok(())
	}
}


struct CpuSpeedStep {
	eist_available: bool,
	eist_enabled: bool,
	eist_locked: bool,
	energy_bias_preference: bool,
	max_pstate: u8,
	is_turbo_pstate: bool,
}

impl CpuSpeedStep {
	const fn new() -> Self {
		CpuSpeedStep {
			eist_available: false,
			eist_enabled: false,
			eist_locked: false,
			energy_bias_preference: false,
			max_pstate: 0,
			is_turbo_pstate: false,
		}
	}

	fn detect_features(&mut self, cpuid: &CpuId) {
		let feature_info = cpuid.get_feature_info().expect("CPUID Feature Info not available!");

		self.eist_available = feature_info.has_eist();
		if !self.eist_available {
			return;
		}

		let misc = unsafe { rdmsr(IA32_MISC_ENABLE) };
		self.eist_enabled = (misc & IA32_MISC_ENABLE_ENHANCED_SPEEDSTEP) > 0;
		self.eist_locked = (misc & IA32_MISC_ENABLE_SPEEDSTEP_LOCK) > 0;
		if !self.eist_enabled || self.eist_locked {
			return;
		}

		self.max_pstate = (unsafe { rdmsr(MSR_PLATFORM_INFO) } >> 8) as u8;
		if (misc & IA32_MISC_ENABLE_TURBO_DISABLE) == 0 {
			let turbo_pstate = unsafe { rdmsr(MSR_TURBO_RATIO_LIMIT) } as u8;
			if turbo_pstate > self.max_pstate {
				self.max_pstate = turbo_pstate;
				self.is_turbo_pstate = true;
			}
		}

		if let Some(thermal_power_info) = cpuid.get_thermal_power_info() {
			self.energy_bias_preference = thermal_power_info.has_energy_bias_pref();
		}
	}

	fn configure(&self) {
		if !self.eist_available || !self.eist_enabled || self.eist_locked {
			return;
		}

		if self.energy_bias_preference {
			unsafe { wrmsr(IA32_ENERGY_PERF_BIAS, 0); }
		}

		let mut perf_ctl_mask = (self.max_pstate as u64) << 8;
		if self.is_turbo_pstate {
			perf_ctl_mask |= 1 << 32;
		}

		unsafe { wrmsr(IA32_PERF_CTL, perf_ctl_mask); }
	}
}

impl fmt::Display for CpuSpeedStep {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if self.eist_available {
			write!(f, "Available, ")?;

			if !self.eist_enabled {
				write!(f, "but disabled")?;
			} else if self.eist_locked {
				write!(f, "but locked")?;
			} else {
				write!(f, "enabled with maximum P-State {}", self.max_pstate)?;
				if self.is_turbo_pstate {
					write!(f, " (Turbo Mode)")?;
				}

				if self.energy_bias_preference {
					write!(f, ", disabled Performance/Energy Bias")?;
				}
			}
		} else {
			write!(f, "Not Available")?;
		}

		Ok(())
	}
}


pub fn detect_features() {
	// Detect CPU features
	let cpuid = CpuId::new();
	let feature_info = cpuid.get_feature_info().expect("CPUID Feature Info not available!");
	let extended_feature_info = cpuid.get_extended_feature_info().expect("CPUID Extended Feature Info not available!");
	let extended_function_info = cpuid.get_extended_function_info().expect("CPUID Extended Function Info not available!");

	unsafe {
		PHYSICAL_ADDRESS_BITS = extended_function_info.physical_address_bits().expect("CPUID Physical Address Bits not available!");
		LINEAR_ADDRESS_BITS = extended_function_info.linear_address_bits().expect("CPUID Linear Address Bits not available!");
		SUPPORTS_1GIB_PAGES = extended_function_info.has_1gib_pages();
		SUPPORTS_AVX = feature_info.has_avx();
		SUPPORTS_FSGSBASE = extended_feature_info.has_fsgsbase();
		SUPPORTS_XSAVE = feature_info.has_xsave();

		if extended_function_info.has_rdtscp() {
			TIMESTAMP_FUNCTION = get_timestamp_rdtscp;
		}

		CPU_SPEEDSTEP.detect_features(&cpuid);
	}
}

pub fn configure() {
	//
	// CR0 CONFIGURATION
	//
	let mut cr0 = unsafe { cr0() };

	// Enable the FPU.
	cr0.insert(CR0_MONITOR_COPROCESSOR | CR0_NUMERIC_ERROR);
	cr0.remove(CR0_EMULATE_COPROCESSOR);

	// Prevent writes to read-only pages in Ring 0.
	cr0.insert(CR0_WRITE_PROTECT);

	// Enable caching.
	cr0.remove(CR0_CACHE_DISABLE | CR0_NOT_WRITE_THROUGH);

	unsafe { cr0_write(cr0); }

	//
	// CR4 CONFIGURATION
	//
	let mut cr4 = unsafe { cr4() };

	// Enable Machine Check Exceptions.
	// No need to check for support here, all x86-64 CPUs support it.
	cr4.insert(CR4_ENABLE_MACHINE_CHECK);

	// Enable full SSE support and indicates that the OS saves SSE context using FXSR.
	// No need to check for support here, all x86-64 CPUs support at least SSE2.
	cr4.insert(CR4_ENABLE_SSE | CR4_UNMASKED_SSE);

	if supports_xsave() {
		// Indicate that the OS saves extended context (AVX, AVX2, MPX, etc.) using XSAVE.
		cr4.insert(CR4_ENABLE_OS_XSAVE);
	}

	// Enable FSGSBASE if available to read and write FS and GS faster.
	if supports_fsgsbase() {
		cr4.insert(CR4_ENABLE_FSGSBASE);

		// Use NOPs to patch out jumps over FSGSBASE usage in entry.asm.
		unsafe {
			ptr::write_bytes(&mut Lpatch0 as *mut u8, 0x90, 2);
			ptr::write_bytes(&mut Lpatch1 as *mut u8, 0x90, 2);
			ptr::write_bytes(&mut Lpatch2 as *mut u8, 0x90, 2);
		}
	}

	unsafe { cr4_write(cr4); }

	//
	// XCR0 CONFIGURATION
	//
	if supports_xsave() {
		// Enable saving the context for all known vector extensions.
		// Must happen after CR4_ENABLE_OS_XSAVE has been set.
		let mut xcr0 = unsafe { xcr0() };
		xcr0.insert(XCR0_FPU_MMX_STATE | XCR0_SSE_STATE);

		if supports_avx() {
			xcr0.insert(XCR0_AVX_STATE);
		}

		unsafe { xcr0_write(xcr0); }
	}

	//
	// MSR CONFIGURATION
	//
	let mut efer = unsafe { rdmsr(IA32_EFER) };

	// Enable support for the EXECUTE_DISABLE paging bit.
	// No need to check for support here, it is always supported in x86-64 long mode.
	efer |= EFER_NXE;
	unsafe { wrmsr(IA32_EFER, efer); }

	// Initialize the FS register, which is later used for Thread-Local Storage.
	unsafe { writefs(0); }

	// Initialize the GS register, which is used for the per_core offset.
	unsafe {
		let size = &percore_end0 as *const u8 as usize - &percore_start as *const u8 as usize;
		let offset = current_boot_id as usize * size;
		writegs(offset);
		wrmsr(IA32_KERNEL_GS_BASE, 0);
	}

	// Initialize the core ID.
	unsafe { __core_id.set_per_core(current_boot_id as u32); }

	//
	// ENHANCED INTEL SPEEDSTEP CONFIGURATION
	//
	unsafe { CPU_SPEEDSTEP.configure(); }
}


pub fn detect_frequency() {
	unsafe { CPU_FREQUENCY.detect(); }
}

pub fn print_information() {
	let cpuid = CpuId::new();
	let extended_function_info = cpuid.get_extended_function_info().expect("CPUID Extended Function Info not available!");
	let brand_string = extended_function_info.processor_brand_string().expect("CPUID Brand String not available!");
	let feature_printer = CpuFeaturePrinter::new(&cpuid);

	info!("");
	info!("=============================== CPU INFORMATION ===============================");
	info!("Model:                  {}", brand_string);
	unsafe {
	info!("Frequency:              {}", CPU_FREQUENCY);
	info!("SpeedStep Technology:   {}", CPU_SPEEDSTEP);
	}
	info!("Features:               {}", feature_printer);
	info!("Physical Address Width: {} bits", get_physical_address_bits());
	info!("Linear Address Width:   {} bits", get_linear_address_bits());
	info!("Supports 1GiB Pages:    {}", if supports_1gib_pages() { "Yes" } else { "No" });
	info!("===============================================================================");
	info!("");
}

#[inline]
pub fn get_linear_address_bits() -> u8 {
	unsafe { LINEAR_ADDRESS_BITS }
}

#[inline]
pub fn get_physical_address_bits() -> u8 {
	unsafe { PHYSICAL_ADDRESS_BITS }
}

#[inline]
pub fn supports_1gib_pages() -> bool {
	unsafe { SUPPORTS_1GIB_PAGES }
}

#[inline]
pub fn supports_avx() -> bool {
	unsafe { SUPPORTS_AVX }
}

#[inline]
pub fn supports_fsgsbase() -> bool {
	unsafe { SUPPORTS_FSGSBASE }
}

#[inline]
pub fn supports_xsave() -> bool {
	unsafe { SUPPORTS_XSAVE }
}

pub fn halt() {
	loop {
		unsafe {
			asm!("hlt" :::: "volatile");
		}
	}
}

#[inline(always)]
pub fn pause() {
	unsafe {
		asm!("pause" :::: "volatile");
	}
}

pub fn update_ticks() {
	unsafe {
		let last_cycles = last_rdtsc.per_core();
		let current_cycles = TIMESTAMP_FUNCTION();
		let cycle_count = abs_diff(last_cycles, current_cycles);
		let tick_count = cycle_count * TIMER_FREQUENCY / (get_cpu_frequency() as u64 * 1_000_000);

		if tick_count > 0 {
			timer_ticks.set_per_core(timer_ticks.per_core() + tick_count);
			last_rdtsc.set_per_core(current_cycles);
		}
	}
}

#[no_mangle]
pub extern "C" fn cpu_detection() -> i32 {
	configure();
	0
}

#[no_mangle]
pub extern "C" fn get_cpu_frequency() -> u32 {
	unsafe { CPU_FREQUENCY.get() as u32 }
}

#[no_mangle]
pub unsafe extern "C" fn fpu_init(fpu_state: *mut XSaveArea) {
	if supports_xsave() {
		ptr::write_bytes(fpu_state, 0, 1);
	} else {
		ptr::write_bytes(&mut (*fpu_state).legacy_region as *mut XSaveLegacyRegion, 0, 1);
	}

	(*fpu_state).legacy_region.fpu_control_word = 0x37f;
	(*fpu_state).legacy_region.mxcsr = 0x1f80;
}

#[no_mangle]
pub unsafe extern "C" fn restore_fpu_state(fpu_state: *const XSaveArea) {
	if supports_xsave() {
		let bitmask: u32 = !0;
		asm!("xrstorq $0" :: "*m"(fpu_state), "{eax}"(bitmask), "{edx}"(bitmask));
	} else {
		asm!("fxrstor $0" :: "*m"(fpu_state));
	}
}

#[no_mangle]
pub unsafe extern "C" fn save_fpu_state(fpu_state: *mut XSaveArea) {
	if supports_xsave() {
		let bitmask: u32 = !0;
		asm!("xsaveq $0" : "=*m"(fpu_state) : "{eax}"(bitmask), "{edx}"(bitmask) : "memory");
	} else {
		asm!("fxsave $0; fnclex" : "=*m"(fpu_state) :: "memory");
	}
}

#[no_mangle]
pub unsafe extern "C" fn readfs() -> usize {
	if supports_fsgsbase() {
		let fs: usize;
		asm!("rdfsbase $0" : "=r"(fs) :: "memory");
		fs
	} else {
		rdmsr(IA32_FS_BASE) as usize
	}
}

#[no_mangle]
pub unsafe extern "C" fn writefs(fs: usize) {
	if supports_fsgsbase() {
		asm!("wrfsbase $0" :: "r"(fs));
	} else {
		wrmsr(IA32_FS_BASE, fs as u64);
	}
}

#[no_mangle]
pub unsafe extern "C" fn writegs(gs: usize) {
	if supports_fsgsbase() {
		asm!("wrgsbase $0" :: "r"(gs));
	} else {
		wrmsr(IA32_GS_BASE, gs as u64);
	}
}

#[inline]
unsafe fn get_timestamp_rdtsc() -> u64 {
	asm!("lfence" ::: "memory");
	let value = rdtsc();
	asm!("lfence" ::: "memory");
	value
}

#[inline]
unsafe fn get_timestamp_rdtscp() -> u64 {
	let value = rdtscp();
	asm!("lfence" ::: "memory");
	value
}

#[inline]
fn abs_diff(a: u64, b: u64) -> u64 {
	if a > b { a - b } else { b - a }
}

#[no_mangle]
pub unsafe extern "C" fn udelay(usecs: u32) {
	let deadline = get_cpu_frequency() as u64 * usecs as u64;
	let start = TIMESTAMP_FUNCTION();

	loop {
		let end = TIMESTAMP_FUNCTION();

		let cycle_count = abs_diff(start, end);
		if cycle_count >= deadline {
			break;
		}

		// If we still have enough cycles left, check if any work can be done in the meantime.
		if deadline - cycle_count > 50000 {
			check_workqueues_in_irqhandler(-1);
		}
	}
}