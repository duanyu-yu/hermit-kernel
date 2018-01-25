// Copyright (c) 2018 Colin Finck, RWTH Aachen University
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

use arch;
use arch::percore::*;
use core::isize;
use errno::*;
use scheduler;
use scheduler::task::Priority;

pub type signal_handler_t = extern "C" fn(i32);
pub type tid_t = u32;


#[no_mangle]
pub extern "C" fn sys_getpid() -> tid_t {
	let core_scheduler = scheduler::get_scheduler(core_id());
	let task = core_scheduler.get_current_task();
	let borrowed = task.borrow();
	borrowed.id.into() as tid_t
}

#[no_mangle]
pub extern "C" fn sys_getprio(id: *const tid_t) -> i32 {
	let core_scheduler = scheduler::get_scheduler(core_id());
	let task = core_scheduler.get_current_task();
	let borrowed = task.borrow();

	if id.is_null() || unsafe {*id} == borrowed.id.into() as u32 {
		borrowed.prio.into() as i32
	} else {
		-EINVAL
	}
}

#[no_mangle]
pub extern "C" fn sys_setprio(id: *const tid_t, prio: i32) -> i32 {
	-ENOSYS
}

#[no_mangle]
pub extern "C" fn sys_exit(arg: i32) -> ! {
	let core_scheduler = scheduler::get_scheduler(core_id());
	core_scheduler.exit(arg);
}

#[no_mangle]
pub extern "C" fn sys_sbrk(incr: isize) -> usize {
	// Get the boundaries of the task heap and verify that they are suitable for sbrk.
	let task_heap_start = arch::mm::virtualmem::task_heap_start();
	let task_heap_end = arch::mm::virtualmem::task_heap_end();
	assert!(task_heap_end <= isize::MAX as usize);

	// Get the heap of the current task on the current core.
	let core_scheduler = scheduler::get_scheduler(core_id());
	let task = core_scheduler.get_current_task();
	let mut borrowed = task.borrow_mut();
	let heap = borrowed.heap.as_mut().expect("Calling sys_sbrk on a task without an associated heap");

	// Adjust the heap of the current task.
	let mut heap_borrowed = heap.borrow_mut();
	assert!(heap_borrowed.start >= task_heap_start, "heap.start {:#X} is not >= task_heap_start {:#X}", heap_borrowed.start, task_heap_start);
	let old_end = heap_borrowed.end;
	heap_borrowed.end = (old_end as isize + incr) as usize;
	assert!(heap_borrowed.end <= task_heap_end, "New heap.end {:#X} is not <= task_heap_end {:#X}", heap_borrowed.end, task_heap_end);

	debug!("Adjusted task heap from {:#X} to {:#X}", old_end, heap_borrowed.end);

	// We're done! The page fault handler will map the new virtual memory area to physical memory
	// as soon as the task accesses it for the first time.
	old_end
}

#[no_mangle]
pub extern "C" fn sys_msleep(ms: u32) {
	panic!("sys_msleep is unimplemented");
}

#[no_mangle]
pub extern "C" fn sys_clone(id: *mut tid_t, func: extern "C" fn(usize), arg: usize) -> i32 {
	let core_scheduler = scheduler::get_scheduler(core_id());
	let task_id = core_scheduler.clone(func, arg);

	if !id.is_null() {
		unsafe { *id = task_id.into() as u32; }
	}

	0
}

#[no_mangle]
pub extern "C" fn sys_yield() {
	panic!("sys_yield is unimplemented");
}

#[no_mangle]
pub extern "C" fn sys_kill(dest: tid_t, signum: i32) -> i32 {
	info!("sys_kill is unimplemented, returning -ENOSYS for killing {} with signal {}", dest, signum);
	-ENOSYS
}

#[no_mangle]
pub extern "C" fn sys_signal(handler: signal_handler_t) -> i32 {
	info!("sys_signal is unimplemented");
	0
}

#[no_mangle]
pub extern "C" fn sys_spawn(id: *mut tid_t, func: extern "C" fn(usize), arg: usize, prio: u8, core_id: u32) -> i32 {
	let core_scheduler = scheduler::get_scheduler(core_id);
	let task_id = core_scheduler.spawn(func, arg, Priority::from(prio), None);

	if !id.is_null() {
		unsafe { *id = task_id.into() as u32; }
	}

	0
}
