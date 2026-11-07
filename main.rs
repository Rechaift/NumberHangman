use std::io::{self, Write as _, StdoutLock};

#[allow(dead_code)]
#[inline(never)]
#[cold]
fn cold() {}

#[allow(dead_code)]
#[inline]
fn likely(b : bool) -> bool {
	if !b { cold(); }
	b
}

#[allow(dead_code)]
#[inline]
fn unlikely(b : bool) -> bool {
	if b { cold() }
	b
}
// rft_term {{{
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
#[allow(unused)]
mod rft_term {
	use std::io::{self, Write, Read, StdinLock};
	use std::process::{Command, Stdio};
	use std::ffi::{c_ushort, c_int, c_uint, c_ulong, c_char, c_uchar, c_void};
	use std::os::unix::io::{RawFd, AsRawFd};
	use std::os::unix::prelude::FromRawFd;
	use std::sync::atomic::{AtomicU16, Ordering};

#[inline(never)]
#[cold]
	fn cold() {}

#[inline]
	fn likely(b : bool) -> bool {
		if !b { cold(); }
		b
	}

#[inline]
	fn unlikely(b : bool) -> bool {
		if b { cold() }
		b
	}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]	
	struct WinSize {
		ws_row: u16,
		ws_col: u16,
		ws_xpixel: u16,
		ws_ypixel: u16,
	}

		fn ioctl(fd: RawFd, request: c_ulong, ...) -> c_int;
	}

	fn get_winsize(fd: RawFd) -> Result<WinSize, c_int> {
		let mut winsize = WinSize::default();

		unsafe {
			if likely(ioctl(fd, TIOCGWINSZ, &mut winsize as *mut WinSize) == 0) {
				Ok(winsize)
			} else {
				Err(-1)
			}
		}
	}

	const EPOLL_CLOEXEC : c_int = 0x80000;
	const EPOLL_CTL_ADD : c_int = 1;
	const EPOLL_CTL_DEL : c_int = 2;
	const EPOLL_CTL_MOD : c_int = 3;
	const EPOLLIN : u32 = 0x00000001;

	#[repr(C)]
	union epoll_data_t {
		ptr: *mut c_void,
		fd: c_int,
		u32: u32,
		u64: u64
	}

	#[repr(C)]
	struct epoll_event {
		events: u32,
		data: epoll_data_t
	}

	unsafe extern "C" {
		fn epoll_create1(flags: c_int) -> c_int;
		fn epoll_ctl(epoll_fd: c_int, op: c_int, fd: c_int, event: *mut epoll_event) -> c_int;
		fn epoll_wait(epoll_fd: c_int, events: *mut epoll_event, event_count: c_int, timeout_ms: c_int) -> c_int;
	}

	fn epoll_create() -> Result<RawFd, String> {
		let fd = unsafe {
			epoll_create1(EPOLL_CLOEXEC)
		};
		if unlikely(fd < 0) {
			return Err("Failed to create epoll!".to_string());
		}
		Ok(fd)
	}

	fn epoll_add(epoll_fd: RawFd, fd: RawFd, events: u32) -> Result<(), String> {
		let mut event = epoll_event {
			events,
			data: epoll_data_t {fd}
		};
		let res = unsafe {
			epoll_ctl(epoll_fd, EPOLL_CTL_ADD, fd, &mut event)
		};
		if unlikely(res < 0) {
			return Err("Failed to add file descriptor to epoll!".to_string());
		}
		Ok(())
	}

	fn epoll_wait_events(epoll_fd: RawFd, events_slice: &mut [epoll_event], timeout_ms: c_int) -> Result<c_int, String> {
		let count = unsafe {
			epoll_wait(epoll_fd, events_slice.as_mut_ptr(), events_slice.len() as c_int, timeout_ms)
		};
		if unlikely(count < 0) {
			return Err("Failed to wait for epoll events".to_string());
		}
		Ok(count)
	}

	type cc_t = c_uchar;
	type tcflag_t = c_uint;

	const NCCS : usize = 19;

	const TCSANOW : c_int = 0;
	const TCSADRAIN : c_int = 1;

	const TCIFLUSH : c_int = 0;

	const ICANON : tcflag_t = 0x00002;
	const ECHO : tcflag_t = 0x00008;

	#[repr(C)]
	struct termios {
		c_iflag: tcflag_t,
		c_oflag: tcflag_t,
		c_cflag: tcflag_t,
		c_lflag: tcflag_t,
		c_line: cc_t,
		c_cc: [cc_t;NCCS]
	}

	unsafe extern "C" {
		fn tcgetattr(fd: c_int, termios_p: *mut termios) -> c_int;
		fn tcsetattr(fd: c_int, optional_actions: c_int, termios_p: *const termios) -> c_int;
		fn tcflush(fd: c_int, queue: c_int) -> c_int;
	}

	fn get_terminal(fd: RawFd) -> Result<termios, String> {
		let mut tty = std::mem::MaybeUninit::<termios>::uninit();
		unsafe {
			if unlikely(tcgetattr(fd, tty.as_mut_ptr()) != 0) {
				return Err("Unable to get terminal attribute".to_string());
			}
		}
		Ok(unsafe { tty.assume_init() })
	}

	fn set_terminal(fd: RawFd, term: &mut termios, lflags: &[tcflag_t], toggle: bool) -> Result<(), String> {
		if toggle {
			for &flag in lflags {
				term.c_lflag |= flag;
			}
		} else {
			for &flag in lflags {
				term.c_lflag &= !flag;
			}
		}

		unsafe {
			if unlikely(tcsetattr(fd, TCSANOW, term) != 0) {
				return Err("Unable to set terminal attribute".to_string());
			}
		}
		Ok(())
	}

	const _KERNEL__NSIG: usize = 64;
	const _NSIG_BPW: usize = 64;
	const _NSIG_WORDS: usize = _KERNEL__NSIG / _NSIG_BPW;

	const SIG_BLOCK: c_int = 0;

	const SIGWINCH: c_int = 28;

	type sigset_t = [u64; _NSIG_WORDS];

	#[repr(C)]
	struct signalfd_siginfo {
		ssi_signo: c_uint,
		ssi_errno: c_int,
		ssi_code: c_int,
		ssi_pid: c_uint,
		ssi_uid: c_uint,
		ssi_fd: c_int,
		ssi_tid: c_uint,
		ssi_band: c_uint,
		ssi_overrun: c_uint,
		ssi_trapno: c_uint,
		ssi_status: c_int,
		ssi_int: c_int,
		ssi_ptr: c_ulong,
		ssi_utime: c_ulong,
		ssi_stime: c_ulong,
		ssi_addr: c_ulong,
		ssi_addr_lsb: c_ushort,
		__pad2: c_ushort,
		ssi_syscall: c_int,
		ssi_call_addr: c_ulong,
		ssi_arch: c_uint,
		__pad: [c_uchar;28]
	}

	unsafe extern "C" {
		fn sigaddset(set: *mut sigset_t, signal: c_int) -> c_int;
		fn sigemptyset(set: *mut sigset_t) -> c_int;
		fn pthread_sigmask(how: c_int, new_set: *const sigset_t, old_set: *mut sigset_t) -> c_int;
		fn signalfd(fd: c_int, mask: *const sigset_t, flags: c_int) -> c_int;
	}

	fn create_signalfd(signal: i32) -> Result<RawFd, String> {
		let mut mask : sigset_t = [0; _NSIG_WORDS];
		unsafe {
			if unlikely(sigemptyset(&mut mask) != 0) {
				return Err("Unable to clear sigset mask".to_string());
			} else if unlikely(sigaddset(&mut mask, signal as c_int) != 0) {
				return Err("Unable to add signal to sigset mask".to_string());
			}
		}

		unsafe {
			if unlikely(pthread_sigmask(SIG_BLOCK, &mask, std::ptr::null_mut()) != 0) {
				return Err("Unable block signal for sigset mask".to_string());
			}
		}

		let sfd = unsafe {
			signalfd(-1, &mask, 0)
		};

		if unlikely(sfd < 0) {
			return Err("Unable to create signal file descriptor".to_string());
		}

		Ok(sfd)
	}

	pub type TermPos = u16;
	pub type TermSize = u16;

	pub enum TermConfig {
		NoCursor,
		EnableMouse
	}

	pub enum TermEvent {
		Key(char),
		Esc(char),
		Unicode(char, u8),
		Click(TermPos, TermPos),
		ScrollUp(TermPos, TermPos),
		ScrollDown(TermPos, TermPos),
		Resize(TermSize, TermSize),
		None
	}

	struct input_info {
		buf : [u8;64],
		len : usize,
		cursor : usize
	}

	pub struct TerminalSetup {
		_term : termios,
		_input_info : input_info,
		_i_events : [epoll_event;2],
		_sigwinch_file : std::fs::File,
		_epoll_fd : RawFd,
		out_fd : RawFd,
		in_fd : RawFd,
		width: TermSize,
		height: TermSize
	}

	impl TerminalSetup {
		pub fn new(conf: &[TermConfig]) -> Result<Self, String> {			
			let in_fd = io::stdin().as_raw_fd();

			let _epoll_fd = epoll_create()?;
			let _sigwinch_fd = create_signalfd(SIGWINCH as i32)?;

			epoll_add(_epoll_fd, in_fd, EPOLLIN)?;
			epoll_add(_epoll_fd, _sigwinch_fd, EPOLLIN)?;

			let out_fd = io::stdout().as_raw_fd();

			let mut width : TermSize = 0;
			let mut height : TermSize = 0;

			if let Ok(winsize) = get_winsize(out_fd) {
				height = winsize.ws_row;
				width = winsize.ws_col;
			} else {
				return Err("Error getting terminal size on startup".to_string());
			}

			let mut _term = get_terminal(in_fd)?;

			if let Err(s) = set_terminal(in_fd, &mut _term, &[ICANON, ECHO], false) {
				if unlikely(std::process::Command::new("stty").args(&["-icanon", "-echo"]).stdin(Stdio::inherit()).output().is_err()) { return Err(s); }
			}

			print!("\x1b[?1049h\x1b[H");
			for i in conf {
				match i {
					TermConfig::NoCursor => print!("\x1b[?25l"),
					TermConfig::EnableMouse => print!("\x1b[?1000h\x1b[?1006l")
				}
			}
			let _ = io::stdout().flush();

			Ok(Self {
				_term,
				_input_info: input_info {
					buf: [0;64],
					len: 0,
					cursor: 0
				},
				_i_events: [epoll_event {
					events: 0,
					data: epoll_data_t { u64: 0 }
				}, epoll_event {
					events: 0,
					data: epoll_data_t { u64: 0}
				}],
				_sigwinch_file: unsafe { std::fs::File::from_raw_fd(_sigwinch_fd) },
				_epoll_fd,
				out_fd,
				in_fd,
				width,
				height
			})
		}

		pub fn pollInput(&mut self, io_in: &mut StdinLock<'_>, timeout: i32) -> TermEvent {
			let mut event = TermEvent::None;
			if let Ok(i) = {
				if self._input_info.cursor < self._input_info.len {
					unsafe {
						self._i_events[0].data.fd = self.in_fd;
					}
					Ok(1)
				} else {
					epoll_wait_events(self._epoll_fd, &mut self._i_events, timeout as c_int)
				}
			}	{
				if i > 0 {
					for ev in 0..i {
						let e_fd = unsafe { self._i_events[ev as usize].data.fd };
						if e_fd == self.in_fd {
							const ESC_STATE : u8 = 1;
							const ESCSEQ_STATE : u8 = 2;
							const MOUSE_STATE : u8 = 3;

							fn skipread(info: &mut input_info, io_in: &mut StdinLock<'_>, mut count : usize) -> usize {
								let mut accumulated : usize = 0;
								while count != 0 {
									if info.cursor == info.len {
										info.len = io_in.read(&mut info.buf).unwrap_or(0);
										info.cursor = 0;
										if info.len == 0 { return accumulated; }
									}
									let diff : usize = info.len - info.cursor;

									if diff >= count {
										info.cursor += count;
										accumulated += count;
										count = 0;
									} else {
										info.cursor += diff;
										accumulated += diff;
										count -= diff;
									}
								}
								accumulated
							}

							fn bulkread(info: &mut input_info, io_in : &mut StdinLock<'_>, payload: &mut [u8], mut count: usize, modifier : i8) -> usize {
								let mut accumulated : usize = 0;
								while count != 0 {
									if info.cursor == info.len {
										info.len = io_in.read(&mut info.buf).unwrap_or(0);
										info.cursor = 0;
										if info.len == 0 { return accumulated; }
									}

									let diff : usize = info.len - info.cursor;
									if diff >= count {
										for i in 0..count {
											payload[accumulated + i] = ((info.buf[info.cursor + i] as i8) + modifier) as u8;
										}
										info.cursor += count;
										accumulated += count;
										count = 0;
									} else {
										for i in 0..diff {
											payload[accumulated + i] = ((info.buf[info.cursor + i] as i8) + modifier) as u8;
										}
										info.cursor += diff;
										accumulated += diff;
										count -= diff;
									}
								}
								accumulated
							}

							fn skipuntil(info: &mut input_info, io_in: &mut StdinLock<'_>, val : u8, not_op : bool, bit : Option<u8>) -> usize {
								let mut accumulated : usize = 0;
								let mask = match bit {
									Some(x) => x,
									None => 0b11111111
								};
								loop {
									if info.cursor == info.len {
										info.len = io_in.read(&mut info.buf).unwrap_or(0);
										info.cursor = 0;
										if info.len == 0 { return accumulated; }
									}
									let diff : usize = info.len - info.cursor;

									if not_op {
										for i in 0..diff {
											if (info.buf[info.cursor + i] & mask) != val {
												info.cursor += i;
												return accumulated;
											}
											accumulated += 1;
										}
									} else {
										for i in 0..diff {
											if (info.buf[info.cursor + i] & mask) == val {
												info.cursor += i;
												return accumulated;
											}
											accumulated += 1;
										}
									}
									info.cursor += diff;
								}
								accumulated
							}

							if self._input_info.cursor == self._input_info.len {
								if let Ok(n) = io_in.read(&mut self._input_info.buf) {
									if n == 1 && self._input_info.buf[0] == b'\x1b' {
										return TermEvent::Esc('\x1b');
									}
									self._input_info.len = n;
									self._input_info.cursor = 0;
								}
							}

							let mut state : u8 = 0;

							while self._input_info.cursor < self._input_info.len {
								match self._input_info.buf[self._input_info.cursor] {
									b'\x1b' => if state != ESC_STATE && state != ESCSEQ_STATE { 
										state = ESC_STATE;
										self._input_info.cursor += 1;
									} else {
										return TermEvent::Esc('\x1b');
									},

									b'[' => if state == ESC_STATE {
										state = ESCSEQ_STATE;
										self._input_info.cursor += 1;
									} else {
										self._input_info.cursor += 1;
										return TermEvent::Key('[')
									},

									b'M' => if state == ESCSEQ_STATE {
										state = MOUSE_STATE;
										self._input_info.cursor += 1;
									} else {
										self._input_info.cursor += 1;
										return TermEvent::Key('M')
									},

									_ => if state == ESC_STATE {
										self._input_info.cursor += 1;
										return TermEvent::Esc('\x1b');
									} else if state == ESCSEQ_STATE {
										self._input_info.cursor += 1;
										return TermEvent::Esc(self._input_info.buf[self._input_info.cursor - 1] as char);
									} else if state == MOUSE_STATE {
										let mcode : u8 = self._input_info.buf[self._input_info.cursor] - 32;
										self._input_info.cursor += 1;
										
										event = match mcode {
											3 => TermEvent::Click(0, 0),
											64 => TermEvent::ScrollUp(0, 0),
											65 => TermEvent::ScrollDown(0, 0),
											_ => TermEvent::None
										};

										match &mut event {
											TermEvent::Click(x, y) | TermEvent::ScrollUp(x, y) | TermEvent::ScrollDown(x, y) => {
												let mut pos : [u8;2] = [0;2];
												bulkread(&mut self._input_info, io_in, &mut pos, 2, -33);
												*x = pos[0] as u16;
												*y = pos[1] as u16;
											},
											_ => {skipread(&mut self._input_info, io_in, 2); continue;}
										}
										return event;
									} else {
										let current : u8 = self._input_info.buf[self._input_info.cursor];
										let ucl : u8 = match current {
											0b00000000..=0b01111111 => 1,
											0b11000000..=0b11011111 => 2,
											0b11100000..=0b11101111 => 3,
											0b11110000..=0b11110111 => 4,
											_ => 0
										};

										if unlikely(ucl == 0) {
											skipuntil(&mut self._input_info, io_in, 0b10000000, true, Some(0b11000000));
											return TermEvent::Unicode('�', 3);
										} else if ucl == 1 {
											self._input_info.cursor += 1;
											return TermEvent::Key(current as char);
										} else {
											self._input_info.cursor += 1;
											let mut uni : [u8;4] = [current, 0, 0, 0];
											bulkread(&mut self._input_info, io_in, &mut uni[1..], (ucl - 1) as usize, 0);
											let unicode : u32 = match ucl {
												2 => (((uni[0] as u32 & 0x1F) << 6) | (uni[1] as u32 & 0x3F)),
												3 => (((uni[0] as u32 & 0x0F) << 12) | ((uni[1] as u32 & 0x3F) << 6) | (uni[2] as u32 & 0x3F)),
												4 => (((uni[0] as u32 & 0x07) << 18) | ((uni[1] as u32 & 0x3F) << 12) | ((uni[2] as u32 & 0x3F) << 6) | (uni[3] as u32 & 0x3F)),
												_ => '�' as u32
											};

											return match unicode {
												0xFE00..=0xFE0F | 0xE0100..=0xE01EF => TermEvent::None,
												_ => TermEvent::Unicode(unsafe {char::from_u32_unchecked(unicode)}, ucl)
											};
										}
									}
								} // End of Match
								if self._input_info.cursor == self._input_info.len {
									self._input_info.len = io_in.read(&mut self._input_info.buf).unwrap_or(0);
									self._input_info.cursor = 0;
									if self._input_info.len == 0 { break; }
								}
							} // End of While	
						} else if e_fd == self._sigwinch_file.as_raw_fd() {
							let mut fsi : signalfd_siginfo = unsafe { std::mem::zeroed() };
							let fsi_sz = std::mem::size_of::<signalfd_siginfo>();
							let buffer = unsafe {
								std::slice::from_raw_parts_mut(&mut fsi as *mut _ as *mut u8, fsi_sz)
							};

							if let Ok(n) = self._sigwinch_file.read(buffer) {
								if n == fsi_sz {
									if fsi.ssi_signo == SIGWINCH as c_uint {
										if let Ok(ws) = get_winsize(self.out_fd) {
											self.width = ws.ws_col;
											self.height = ws.ws_row;
											event = TermEvent::Resize(self.width, self.height);
										}
									}
								} else {
									panic!("\033[?1049l\033[1;31mUnexpected but very dangerous error has occured when trying to read signalfd. The program will quit abruptly as it is dangerous to continue.\033[0m");
								}
							}
						}
					}
				}
			}
			event
		}

		pub fn getSize(&self) -> (TermSize, TermSize) {
			(self.width, self.height)
		}
	}

	impl Drop for TerminalSetup {
		fn drop(&mut self) {
			print!("\x1b[?25h\x1b[?1000l\x1b[?1002l\x1b[?1049l");
			let _ = io::stdout().flush();

			unsafe {
				tcflush(self.in_fd, TCIFLUSH);
			}

			if let Err(e) = set_terminal(self.in_fd, &mut self._term, &[ICANON, ECHO], true) {
				eprintln!("Error when setting terminal back to normal:\n{}\n", e);
				eprintln!("Attempting to set terminal back with Command(\"stty\")");
				std::process::Command::new("stty").args(&["icanon", "echo"]).stdin(Stdio::inherit()).output().expect("Setting Command(\"stty\") failed.\nTerminal is stuck in raw state.\nYou can enable icanon and echo by yourself.\nTry typing:\n\"stty icanon echo\"\nor\n\"stty sane\"");
			}
		}
	}
}
// }}}
// Random {{{
#[allow(unused)]
mod rft_rand {
	use std::io::{self, Read};
	use std::fs::File;
	use std::path::Path;
	use std::os::unix::ffi::OsStrExt;
	use std::ffi::OsStr;
	use std::time::{SystemTime, Instant, UNIX_EPOCH};

	const URANDOM_PATH : &[u8] = b"/dev/urandom\0";

	pub enum RandErr {
		CStrErr,
		PathErr,
		FileOpenErr,
		ReadErr
	}

	pub fn random_seed() -> u64 {
		fn time_seed() -> u64 {
			let wall = SystemTime::now()
				.duration_since(UNIX_EPOCH)
				.unwrap_or_default()
				.as_nanos() as u64;
			wall.wrapping_add(Instant::now().elapsed().as_nanos() as u64)
		}

		let path = Path::new(OsStr::from_bytes(URANDOM_PATH));

		if let Ok(mut urandom) = File::open(path) {

			let mut seed : [u8;8] = [0u8;8];

			if let Err(_) = urandom.read_exact(&mut seed) {
				time_seed()
			} else {
				(seed[0] as u64) | ((seed[1] as u64) << 7) |
					((seed[2] as u64) << 15) | ((seed[3] as u64) << 23) |
					((seed[4] as u64) << 31) | ((seed[5] as u64) << 39) |
					((seed[6] as u64) << 47) | ((seed[7] as u64) << 55)
			}
		} else {
			time_seed()
		}
	}

	pub struct Random {
		seed: u64,
		state: [u64;4],
	}

	impl Random {
		pub fn new(seed: u64) -> Self {
			let mut state: [u64;4] = [0, 0, 0, 0];

			const GOLDEN_RATIO: u64 = 0x9e3779b97f4a7c15;
			const A: u64 = 0xbf58476d1ce4e5b9;
			const B: u64 = 0x94d049bb133111eb;
			let mut temp_st: u64 = seed;
			for i in 0..4 {
				temp_st = temp_st.wrapping_add(GOLDEN_RATIO);
				let mut x = temp_st;
				x = (x ^ (x >> 30)).wrapping_mul(A);
				x = (x ^ (x >> 27)).wrapping_mul(B);
				state[i] = x ^ (x >> 31);
			}

			Self {
				seed,
				state,
			}
		}

		pub fn next(&mut self) -> u64 {
			let result = self.state[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);

			let t = self.state[1] << 17;
			self.state[2] ^= self.state[0];
			self.state[3] ^= self.state[1];
			self.state[1] ^= self.state[2];
			self.state[0] ^= self.state[3];
			self.state[2] ^= t;
			self.state[3] = self.state[3].rotate_left(45);

			result
		}

		pub fn jump(&mut self) {
			const JUMP: [u64; 4] = [
				0x180ec6d33cfd0aba,
				0xd5a61266f0c9392c,
				0xa9582618e03fc9aa,
				0x39abdc4529b1661c,
			];

			let mut new_state = [0; 4];
			for &jump_val in &JUMP {
				for b in 0..64 {
					if (jump_val >> b) & 1 != 0 {
						new_state.iter_mut().zip(&self.state).for_each(|(n, s)| *n ^= s);
					}
					self.next(); // Advance state
				}
			}
			self.state = new_state;
		}

		pub fn range(&mut self, low: u64, high: u64) -> u64 {
			let range = high - low;
			if range == 0 {
				return low;
			}

			let mut x = self.next();
			let mut m = (x as u128).wrapping_mul(range as u128);
			let mut l = m as u64;

			while l < range {
				x = self.next();
				m = (x as u128).wrapping_mul(range as u128);
				l = m as u64;
			}

			low + (m >> 64) as u64
		}

		pub fn chance(&mut self, bias: f64) -> bool {
			debug_assert!((0.0..=1.0).contains(&bias));

			let threshold = (bias * (1u64 << 53) as f64) as u64;
			(self.next() >> 11) < threshold
		}

		fn float(&mut self) -> f64 {
			f64::from_bits((self.next() >> 11) | 0x3FF0000000000000) - 1.0
		}

		pub fn weighted_choice<T: Copy>(&mut self, items: &[(T, f64)]) -> T {
			let total: f64 = items.iter().map(|(_, weight)| weight).sum();
			let mut r = self.float() * total;

			for (item, weight) in items {
				r -= weight;
				if r <= 0.0 {
					return *item;
				}
			}

			items.last().unwrap().0
		}

		pub fn shuffle<T>(&mut self, items: &mut [T]) {
			for i in (1..items.len()).rev() {
				let j = self.range(0, i as u64 + 1) as usize;
				items.swap(i, j);
			}
		}
	}
}
// }}}

use rft_term::TermEvent;
use rft_rand::Random;

type DataPos = u16;
type DataSize = u16;
type DataNode = (DataPos, DataPos, DataSize, DataSize);

const S_HLINE_FULL : [u8;3usize * u8::MAX as usize] = {
	let mut line : [u8;3usize * u8::MAX as usize] = [0u8;3usize * u8::MAX as usize];
	let mut i = 0;
	while i < 3usize * u8::MAX as usize {
		line[i] = 226;
		line[i + 1] = 148;
		line[i + 2] = 128;
		i += 3;
	}
	line
};
const DB_HLINE_FULL : [u8;3usize * u8::MAX as usize] = {
	let mut line : [u8;3usize * u8::MAX as usize] = [0u8;3usize * u8::MAX as usize];
	let mut i = 0;
	while i < 3usize * u8::MAX as usize {
		line[i] = 226;
		line[i + 1] = 149;
		line[i + 2] = 144;
		i += 3;
	}
	line
};

#[inline]
fn is_point_innode(x: DataPos, y: DataPos, node: DataNode) -> bool {
	x >= node.0 && x < node.0 + node.2 && y >= node.1 && y < node.1 + node.3
}

enum ContextCommand {
	Switch(u8),
	Exit,
	None
}

struct ContextData {
	width: DataSize,
	height: DataSize,
	difficulty: u8,
	first_time: bool
}

trait ContextTrait {
	fn input(&mut self, e: TermEvent, data: &mut ContextData) -> ContextCommand;
	fn render(&self, out: &mut StdoutLock<'_>, data: &ContextData);
	fn switch_in(&mut self, _ctx: u8, _data: &mut ContextData) {}
	fn switch_out(&mut self, _ctx: u8, _data: &mut ContextData) {}
}

struct TitleContext {
	node: [DataNode;6]
}

impl TitleContext {
	fn new(w: DataSize, _h: DataSize) -> Self {
		Self {
			node: [
				((w/2).saturating_sub(6),0,u16::MAX,3),
				(0,u16::MAX,18,3),
				(0,4,16,3),
				(0,8,15,3),
				(0,12,15,3),
				(w.saturating_sub(22),6,19,8)
			],
		}
	}
}

impl ContextTrait for TitleContext {
	fn input(&mut self, e: TermEvent, data: &mut ContextData) -> ContextCommand {
		let _ = data;
		match e {
			TermEvent::Key('q') => ContextCommand::Exit,
			TermEvent::Click(x, y) => {
				if is_point_innode(x, y, self.node[1]) {
					ContextCommand::Switch(2)
				}
				else if is_point_innode(x, y, self.node[2]) {
					ContextCommand::Switch(1)
				}
				else if is_point_innode(x, y, self.node[3]) {
					ContextCommand::Switch(3)
				}
				else if is_point_innode(x, y, self.node[4]) {
					ContextCommand::Exit
				} else {
					ContextCommand::None
				}
			},
			TermEvent::Resize(w, _) => {
				self.node[0].0 = (w/2).saturating_sub(6);
				self.node[5].0 = w.saturating_sub(22);
				ContextCommand::None
			}
			_ => ContextCommand::None
		}
	}

	fn render(&self, out: &mut StdoutLock<'_>, data: &ContextData) {
		let _ = data;
		{
			let _ = out.write_all(b"\x1b[H");
			let _ = out.write_all(&DB_HLINE_FULL[..3usize * data.width as usize]);
			let _ = write!(out, "\x1b[2;{}H\x1b[1;33mNumber Hangman\x1b[0m\n", self.node[0].0 + 1);
			let _ = out.write_all(&DB_HLINE_FULL[..3usize * data.width as usize]);
		}
		let _ = if data.first_time {
			write!(out, concat!(
					"\x1b[5H",
					"───────────────┐\n",
					"     \x1b[1mStart\x1b[0m     │\n",
					"───────────────┘",
					"\x1b[9H",
					"──────────────┐\n",
					"     \x1b[92;1mHelp\x1b[0m     │\n",
					"──────────────┘",
					"\x1b[13H",
					"──────────────┐\n",
					"     \x1b[91;1mExit\x1b[0m     │\n",
					"──────────────┘"))
		} else {
			write!(out, concat!(
					"\x1b[5H",
					"──────────────────┐\n",
					"     \x1b[96;1mContinue\x1b[0m     │\n",
					"──────────────────┘",
					"\x1b[9H",
					"───────────────┐\n",
					"     \x1b[1mStart\x1b[0m     │\n",
					"───────────────┘",
					"\x1b[13H",
					"──────────────┐\n",
					"     \x1b[92;1mHelp\x1b[0m     │\n",
					"──────────────┘",
					"\x1b[17H",
					"──────────────┐\n",
					"     \x1b[91;1mExit\x1b[0m     │\n",
					"──────────────┘"))
		};
		let _ = write!(out, concat!(
				"\x1b[7;{}H",
				" Made by  \x1b[91;1mRechaift\x1b[0m ",
				"\x1b[19D\x1b[B",
				"\x1b[33;1mThanks For Playing!\x1b[0m",
				"\x1b[19D\x1b[2B",
				"        ╭─╮\x1b[11D\x1b[B",
				"        ╰─╯╱\x1b[12D\x1b[B",
				"        ╱│\x1b[10D\x1b[B",
				"         │\x1b[10D\x1b[B",
				"        ╱ ╲\x1b[11D\x1b[B"
				), self.node[5].0);
	}

	fn switch_in(&mut self, _ctx: u8, data: &mut ContextData) {
		if data.first_time == false {
			self.node[1].1 = 4;
			self.node[2].1 = 8;
			self.node[3].1 = 12;
			self.node[4].1 = 16;
		}
	}
}

struct DifficultyContext {
	node: [DataNode;5]
}

impl DifficultyContext {
	fn new(w: DataSize, h: DataSize) -> Self {
		let middle = w/2;
		let sb_mid = middle.saturating_sub(7);
		Self {
			node: [
				(middle.saturating_sub(const {((*b"Select Difficulty").len() as u16) / 2}), 1, const {(*b"Select Difficulty").len() as u16}, 1),
				(sb_mid, 4, 14, 3),
				(sb_mid, 8, 14, 3),
				(sb_mid, 12, 14, 3),
				(w.saturating_sub(11), h.saturating_sub(5), 11, 3)
			]
		}
	}
}

impl ContextTrait for DifficultyContext {
	fn input(&mut self, e: TermEvent, data: &mut ContextData) -> ContextCommand {
		match e {
			TermEvent::Click(x, y) => {
				for i in 1..4 {
					if is_point_innode(x, y, self.node[i]) {
						data.difficulty = 3 + (((i as u8) - 1) * 2);
						return ContextCommand::Switch(2);
					}
				}
				if is_point_innode(x, y, self.node[4]) {
					ContextCommand::Switch(0)
				} else {
					ContextCommand::None
				}
			},
			TermEvent::Resize(w, h) => {
				let middle = w/2;
				let sb_mid = middle.saturating_sub(7);
				self.node[0].0 = middle.saturating_sub(const {((*b"Select Difficulty").len() as u16) / 2});
				self.node[1].0 = sb_mid;
				self.node[2].0 = sb_mid;
				self.node[3].0 = sb_mid;
				self.node[4].0 = w.saturating_sub(11);
				self.node[4].1 = h.saturating_sub(5);
				ContextCommand::None
			},
			_ => ContextCommand::None
		}
	}

	fn render(&self, out: &mut StdoutLock<'_>, _data: &ContextData) {
		let _ = write!(out, concat!(
				"\x1b[1;{0}H",
				"Select Difficulty",
				"\x1b[5;{1}H",
				"┌────────────┐",
				"\x1b[14D\x1b[B",
				"│    \x1b[32;1mEasy\x1b[0m    │",
				"\x1b[14D\x1b[B",
				"└────────────┘",
				"\x1b[9;{1}H",
				"┌────────────┐",
				"\x1b[14D\x1b[B",
				"│   \x1b[1mNormal\x1b[0m   │",
				"\x1b[14D\x1b[B",
				"└────────────┘",
				"\x1b[13;{1}H",
				"┌────────────┐",
				"\x1b[14D\x1b[B",
				"│    \x1b[31;1mHard\x1b[0m    │",
				"\x1b[14D\x1b[B",
				"└────────────┘",
				"\x1b[{2};{3}H",
				"┌──────────",
				"\x1b[10D\x1b[B",
				"│   Back   ",
				"\x1b[10D\x1b[B",
				"└──────────"
					),
					self.node[0].0 + 1,
					self.node[1].0 + 1,
					self.node[4].1 + 1, self.node[4].0 + 1);
	}

	fn switch_out(&mut self, ctx: u8, data: &mut ContextData) {
		if ctx == 2 {
			data.first_time = true;
		}
	}
}

struct GameContext {
	rand: Random,
	number_node: [(DataNode, char, i8);7],
	node: [DataNode;12],
	scratchpad: ([char;3], u8),
	attempted_node: [u8;7],
	answer: [u8;7],
	answered: u8,
	selected: u8,
	attempt: u8,
	game_result: i8
}

impl GameContext {
	const BOX_SIZE : u16 = 8;
	const BOX_GAP : u16 = 3;
	fn new(w: DataSize, h: DataSize) -> Self {
		let middle = w/2;
		let center = h/2;
		let popup_x = middle.saturating_sub(19/2);
		let popup_y = center.saturating_sub(10/2);
		let segment = w/3;
		let segend = w.saturating_sub(segment);
		Self {
			rand: Random::new(rft_rand::random_seed()),
			number_node: [((0,0,0,0),' ', -1);7],
			node: [
				(2, h.saturating_sub(5 + 12), 12, 12),
				(0, h.saturating_sub(3), u16::MAX, 1),
				(popup_x, popup_y, 19, 10),
				(popup_x + 1, popup_y + 3, 8, 3),
				(popup_x + 10, popup_y + 3, 8, 3),
				(popup_x + 4, popup_y + 6, 11, 3),
				(0, 0, segment, 4),
				(segment, 0, w.saturating_sub(segend).saturating_sub(segment), 4),
				(segend, 0, w.saturating_sub(segend), 4),
				(w.saturating_sub(17), 5, 21, 4),
				(w.saturating_sub(8), 10, 8, 2),
				(w.saturating_sub(8), 13, 8, 2)
			],
			scratchpad: ([' ';3], 3),
			attempted_node: [0u8;7],
			answer: [10;7],
			answered: 0,
			selected: u8::MAX,
			attempt: 0,
			game_result: -1
		}
	}

	fn restart(&mut self, data: &ContextData) {
		self.number_node = [((0, 0, 0, 0), ' ', -1);7];
		self.scratchpad = ([' ';3], 3);
		self.attempted_node = [0u8;7];
		self.answer = [10;7];
		self.answered = 0;
		self.selected = u8::MAX;
		self.attempt = 0;
		self.game_result = -1;
		{
			let start = (data.width/2).saturating_sub(((Self::BOX_SIZE * data.difficulty as u16)/2).saturating_sub(Self::BOX_GAP));
			for i in 0..data.difficulty as usize {
				self.number_node[i].0 = (start + (Self::BOX_SIZE * i as u16), data.height.saturating_sub(2), 5, 3);

				self.answer[i] = self.rand.range(0, 9) as u8;
			}
		}
	}
}

impl ContextTrait for GameContext {
	fn input(&mut self, e: TermEvent, data: &mut ContextData) -> ContextCommand {
		let _ = data;
		match e {
			TermEvent::Esc('\x1b') => ContextCommand::Switch(0),
			TermEvent::Key(c) => {
				if (c as u8) >= b'0' && (c as u8) <= b'9' {
					if self.selected < data.difficulty && self.number_node[self.selected as usize].2 != 1 {
						self.number_node[self.selected as usize].1 = c;
						if self.answer[self.selected as usize] + b'0' == c as u8 {
							self.number_node[self.selected as usize].2 = 1;
							self.answered += 1;
							if self.answered == data.difficulty {
								self.game_result = 1;
							}
						}
						else {
							self.number_node[self.selected as usize].2 = 0;
							self.attempt += 1;
							if self.attempt == 10 {
								self.game_result = 0;
							}
							self.attempted_node[self.selected as usize] += 1;
							
							if self.scratchpad.1 != 0 {
								if let false = { 
									let mut e = false;
									for i in 0..data.difficulty as usize {
										if self.answer[i] + b'0' == c as u8 {
											e = true;
											break;
										}
									}
									for i in 0..3usize - self.scratchpad.1 as usize {
										if self.scratchpad.0[i] == c {
											e = true;
											break;
										}
									}
									e
								} {
									self.scratchpad.0[3usize - self.scratchpad.1 as usize] = c;
									self.scratchpad.1 -= 1;
								}
							}
						}
					}
				}
				ContextCommand::None
			},
			TermEvent::Click(x, y) => {
				if y > data.height.saturating_sub(3) && self.game_result == -1 {
					for i in 0u8..data.difficulty {
						if is_point_innode(x, y, self.number_node[i as usize].0) {
							self.selected = i;
							return ContextCommand::None;
						}
					}
				} else if self.game_result != -1 {
					if is_point_innode(x, y, self.node[3]) {
						return ContextCommand::Switch(1);
					} else if is_point_innode(x, y, self.node[4]) {
						return ContextCommand::Switch(0);
					} else if is_point_innode(x, y, self.node[5]) {
						self.restart(data);
					}
				} else {
					if is_point_innode(x, y, self.node[6]) {
						return ContextCommand::Switch(1);
					} else if is_point_innode(x, y, self.node[7]) {
						self.attempt = 10;
						self.game_result = 0;
					} else if is_point_innode(x, y, self.node[8]) {
						return ContextCommand::Switch(0);
					}
				}
				ContextCommand::None
			},
			TermEvent::Resize(w, h) => {
				let middle = w/2;
				let center = h/2;
				let popup_x = middle.saturating_sub(19/2);
				let popup_y = center.saturating_sub(10/2);
				let segment = w/3;
				let segend = w.saturating_sub(segment);
				let start = middle.saturating_sub(((Self::BOX_SIZE*data.difficulty as u16)/2).saturating_sub(Self::BOX_GAP));
				let box_vstart = h.saturating_sub(2);
				for i in 0u16..data.difficulty as u16 {
					self.number_node[i as usize].0.0 = start + (Self::BOX_SIZE * i);
					self.number_node[i as usize].0.1 = box_vstart;
				}
				self.node[0].1 = h.saturating_sub(5 + 12);
				self.node[1].1 = h.saturating_sub(3);
				self.node[2].0 = popup_x;
				self.node[2].1 = popup_y;

				self.node[3].0 = popup_x + 1;
				self.node[3].1 = popup_y + 3;
				self.node[4].0 = popup_x + 10;
				self.node[4].1 = popup_y + 3;
				self.node[5].0 = popup_x + 4;
				self.node[5].1 = popup_y + 6;

				self.node[6].2 = segment;
				self.node[7].0 = segment;
				self.node[7].2 = segend.saturating_sub(segment);
				self.node[8].0 = segend;
				self.node[8].2 = w.saturating_sub(segend);

				self.node[9].0 = w.saturating_sub(17);
				self.node[10].0 = w.saturating_sub(8);
				self.node[11].0 = w.saturating_sub(8);
				ContextCommand::None
			}
			_ => ContextCommand::None
		}
	}

	fn render(&self, out: &mut StdoutLock<'_>, data: &ContextData) {
		{
			let segment = data.width/3;
			let segend = data.width.saturating_sub(segment);
			let change = (segment/2).saturating_sub(3);
			let back = segend + (data.width.saturating_sub(segend)/2).saturating_sub(1);
			let end = segment + (segend.saturating_sub(segment)/2).saturating_sub(3/2);

			let _ = write!(out, "\x1b[4H{}", if self.game_result == -1 { "" } else { "\x1b[38;2;128;128;128m" });
			let _ = out.write_all(&S_HLINE_FULL[..3usize * data.width as usize]);
			let _ = write!(out, concat!(
					"\x1b[H",
					"\x1b[1;{0}H│\x1b[1;{1}H│",
					"\x1b[2;{2}HChange\x1b[2;{0}H│",
					"\x1b[2;{3}H{5}End{6}\x1b[2;{1}H│",
					"\x1b[2;{4}HBack",
					"\x1b[3;{0}H│\x1b[3;{1}H│\n",
					"\x1b[4;{0}H┴\x1b[4;{1}H┴\x1b[0m"),
					segment + 1, segend, change + 1, end + 1, back + 1,
					if self.game_result == -1 { "\x1b[31;1m" } else { "\x1b[38;2;128;128;128m" },
					if self.game_result == -1 { "\x1b[0m" } else { "" });
			let _ = write!(out, concat!(
					"\x1b[6;{}H",
					"   Scratch Pad   ",
					"\x1b[17D\x1b[B",
					"╔═══╗ ╔═══╗ ╔═══╗",
					"\x1b[17D\x1b[B",
					"║ {} ║ ║ {} ║ ║ {} ║",
					"\x1b[17D\x1b[B",
					"╚═══╝ ╚═══╝ ╚═══╝"
					),
					self.node[9].0,
					self.scratchpad.0[0],
					self.scratchpad.0[1],
					self.scratchpad.0[2]);
		}

		let _ = write!(out, "\x1b[{};3H", self.node[0].1);
		let _ = match self.attempt {
			0 => Ok(()),
			1 => write!(out, concat!(
					"\x1b[10B",
					"║  ║\x1b[4D\x1b[B",
					"╚══╝"
			)),
			2 => write!(out, concat!(
					" ┌┐\x1b[3D\x1b[B",
					" │\x1b[2D\x1b[B",
					" ││\x1b[3D\x1b[B",
					" ││\x1b[3D\x1b[B",
					" ││\x1b[3D\x1b[B",
					" ││\x1b[3D\x1b[B",
					" ││\x1b[3D\x1b[B",
					" ││\x1b[3D\x1b[B",
					" ││\x1b[3D\x1b[B",
					" ││\x1b[3D\x1b[B",
					"║││║\x1b[4D\x1b[B",
					"╚══╝"
			)),
			3 => write!(out, concat!(		
					" ┌┐\x1b[3D\x1b[B",
					" │════════╗\x1b[11D\x1b[B",
					" ││\x1b[7C│\x1b[11D\x1b[B",
					" ││\x1b[7C│\x1b[11D\x1b[B",
					" ││\x1b[3D\x1b[B",
					" ││\x1b[3D\x1b[B",
					" ││\x1b[3D\x1b[B",
					" ││\x1b[3D\x1b[B",
					" ││\x1b[3D\x1b[B",
					" ││\x1b[3D\x1b[B",
					"║││║\x1b[4D\x1b[B",
					"╚══╝"
			)),
			4 => write!(out, concat!(
					" ┌┐\x1b[3D\x1b[B",
					" │════════╗\x1b[11D\x1b[B",
					" ││\x1b[7C│\x1b[11D\x1b[B",
					" ││\x1b[7C│\x1b[11D\x1b[B",
					" ││\x1b[6C╭┴╮\x1b[12D\x1b[B",
					" ││\x1b[6C╰┬╯\x1b[12D\x1b[B",
					" ││\x1b[3D\x1b[B",
					" ││\x1b[3D\x1b[B",
					" ││\x1b[3D\x1b[B",
					" ││\x1b[3D\x1b[B",
					"║││║\x1b[4D\x1b[B",
					"╚══╝"
			)),
			5 => write!(out, concat!(
					" ┌┐\x1b[3D\x1b[B",
					" │════════╗\x1b[11D\x1b[B",
					" ││\x1b[7C│\x1b[11D\x1b[B",
					" ││\x1b[7C│\x1b[11D\x1b[B",
					" ││\x1b[6C╭┴╮\x1b[12D\x1b[B",
					" ││\x1b[6C╰┬╯\x1b[12D\x1b[B",
					" ││\x1b[7C│\x1b[11D\x1b[B",
					" ││\x1b[3D\x1b[B",
					" ││\x1b[3D\x1b[B",
					" ││\x1b[3D\x1b[B",
					"║││║\x1b[4D\x1b[B",
					"╚══╝"
			)),
			6 => write!(out, concat!(
					" ┌┐\x1b[3D\x1b[B",
					" │════════╗\x1b[11D\x1b[B",
					" ││\x1b[7C│\x1b[11D\x1b[B",
					" ││\x1b[7C│\x1b[11D\x1b[B",
					" ││\x1b[6C╭┴╮\x1b[12D\x1b[B",
					" ││\x1b[6C╰┬╯\x1b[12D\x1b[B",
					" ││\x1b[6C╱│\x1b[11D\x1b[B",
					" ││\x1b[3D\x1b[B",
					" ││\x1b[3D\x1b[B",
					" ││\x1b[3D\x1b[B",
					"║││║\x1b[4D\x1b[B",
					"╚══╝"
			)),
			7 => write!(out, concat!(	
					" ┌┐\x1b[3D\x1b[B",
					" │════════╗\x1b[11D\x1b[B",
					" ││\x1b[7C│\x1b[11D\x1b[B",
					" ││\x1b[7C│\x1b[11D\x1b[B",
					" ││\x1b[6C╭┴╮\x1b[12D\x1b[B",
					" ││\x1b[6C╰┬╯\x1b[12D\x1b[B",
					" ││\x1b[6C╱│╲\x1b[12D\x1b[B",
					" ││\x1b[3D\x1b[B",
					" ││\x1b[3D\x1b[B",
					" ││\x1b[3D\x1b[B",
					"║││║\x1b[4D\x1b[B",
					"╚══╝"
			)),
			8 => write!(out, concat!(
					" ┌┐\x1b[3D\x1b[B",
					" │════════╗\x1b[11D\x1b[B",
					" ││\x1b[7C│\x1b[11D\x1b[B",
					" ││\x1b[7C│\x1b[11D\x1b[B",
					" ││\x1b[6C╭┴╮\x1b[12D\x1b[B",
					" ││\x1b[6C╰┬╯\x1b[12D\x1b[B",
					" ││\x1b[6C╱│╲\x1b[12D\x1b[B",
					" ││\x1b[7C│\x1b[11D\x1b[B",
					" ││\x1b[3D\x1b[B",
					" ││\x1b[3D\x1b[B",
					"║││║\x1b[4D\x1b[B",
					"╚══╝"
			)),
			9 => write!(out, concat!(
					" ┌┐\x1b[3D\x1b[B",
					" │════════╗\x1b[11D\x1b[B",
					" ││\x1b[7C│\x1b[11D\x1b[B",
					" ││\x1b[7C│\x1b[11D\x1b[B",	
					" ││\x1b[6C╭┴╮\x1b[12D\x1b[B",
					" ││\x1b[6C╰┬╯\x1b[12D\x1b[B",
					" ││\x1b[6C╱│╲\x1b[12D\x1b[B",
					" ││\x1b[7C│\x1b[11D\x1b[B",
					" ││\x1b[6C╱\x1b[10D\x1b[B",
					" ││\x1b[3D\x1b[B",
					"║││║\x1b[4D\x1b[B",
					"╚══╝"
			)),
			_ => write!(out, concat!(
					" ┌┐\x1b[3D\x1b[B",
					" │════════╗\x1b[11D\x1b[B",
					" ││\x1b[7C│\x1b[11D\x1b[B",
					" ││\x1b[7C│\x1b[11D\x1b[B",
					" ││\x1b[6C╭┴╮\x1b[12D\x1b[B",
					" ││\x1b[6C╰┬╯\x1b[12D\x1b[B",
					" ││\x1b[6C╱│╲\x1b[12D\x1b[B",
					" ││\x1b[7C│\x1b[11D\x1b[B",
					" ││\x1b[6C╱ ╲\x1b[12D\x1b[B",
					" ││\x1b[3D\x1b[B",
					"║││║\x1b[4D\x1b[B",
					"╚══╝"
			))
		};

		let _ = write!(out, "\x1b[{}H", self.node[1].1);
		let _ = out.write_all(&DB_HLINE_FULL[..3usize * data.width as usize]);
		if likely(self.game_result == -1) {
			for i in 0..data.difficulty as usize {
				if unlikely(self.selected == i as u8) {
					if self.attempted_node[i] > 0 {
						let _ = write!(out, concat!(
								"\x1b[11;{}H",
								"Hot/Cold",
								"\x1b[8D\x1b[B",
								" {} "),
								self.node[10].0,
								{
									let a = self.answer[i];
									let b = (self.number_node[i].1 as u8) - b'0';
									if (b > a && b - a < 3) || (b < a && a - b < 3) {
										"\x1b[31;1m Hot  \x1b[0m"
									} else if (b > a && b - a < 5) || (b < a && a - b < 5) {
										" Warm "
									} else if a == b {
										"\x1b[33;1mSolved\x1b[0m"
									} else {
										"\x1b[36;1m Cold \x1b[0m"
									}
								}
						);
					}
					if self.attempted_node[i] > 1 {
						let _ = write!(out, concat!(
								"\x1b[14;{}H",
								"Odd/Even",
								"\x1b[8D\x1b[B",
								" {} "),
								self.node[11].0,
								if self.number_node[i].2 == 1 { "\x1b[33;1mSolved\x1b[0m" } else if self.answer[i] % 2 == 0 { " Even " } else { " Odd  " }
							);
					}
					let _ = write!(out, concat!(
							"\x1b[{0};{3}H",
							"\x1b[33;1m╔═══╗\x1b[0m",
							"\x1b[{1};{3}H",
							"\x1b[33;1m║\x1b[0m {5}{4}\x1b[0m \x1b[33;1m║\x1b[0m",
							"\x1b[{2};{3}H",
							"\x1b[33;1m╚═══╝\x1b[0m"),
							self.number_node[i].0.1,
							self.number_node[i].0.1 + 1,
							self.number_node[i].0.1 + 2,
							self.number_node[i].0.0,
							self.number_node[i].1,
							match self.number_node[i].2 {
								1 => "\x1b[32;1m",
								0 => "\x1b[31;1m",
								_ => ""
							});
				} else {
					let _ = write!(out, concat!(
							"\x1b[{0};{3}H",
							"┌───┐",
							"\x1b[{1};{3}H",
							"│ {5}{4}\x1b[0m │",
							"\x1b[{2};{3}H",
							"└───┘"),
							self.number_node[i].0.1,
							self.number_node[i].0.1 + 1,
							self.number_node[i].0.1 + 2,
							self.number_node[i].0.0,
							self.number_node[i].1,
							match self.number_node[i].2 {
								1 => "\x1b[32;1m",
								0 => "\x1b[31;1m",
								_ => ""
							});
				}
			}
			if self.number_node[0].2 == 1 && self.number_node[1].2 != 1 {
				let _ = write!(out, "\x1b[{};{}H{}",
						self.number_node[0].0.1 + 1,
						self.number_node[0].0.0 + self.number_node[0].0.2 + 1,
						if self.answer[0] > self.answer[1] { '-' } else if self.answer[0] < self.answer[1] { '+' } else { '=' }
					);
			}
			{
				let diffi = (data.difficulty - 1) as usize;
				if self.number_node[diffi].2 == 1 && self.number_node[diffi - 1].2 != 1 {
					let _ = write!(out, "\x1b[{};{}H{}",
						self.number_node[diffi].0.1 + 1,
						self.number_node[diffi].0.0 - 2,
						if self.answer[diffi] > self.answer[diffi - 1] { '-' } else if self.answer[diffi] < self.answer[diffi - 1] { '+' } else { '=' }
					);
				}
			}
			for i in 1..(data.difficulty as usize) - 1usize {
				if self.number_node[i].2 == 1 && self.number_node[i - 1].2 != 1 {
					let _ = write!(out, "\x1b[{};{}H{}",
						self.number_node[i].0.1 + 1,
						self.number_node[i].0.0 - 2,
						if self.answer[i] > self.answer[i - 1] { '-' } else if self.answer[i] < self.answer[i - 1] { '+' } else { '=' }
					);
				}
				if self.number_node[i].2 == 1 && self.number_node[i + 1].2 != 1 {
					let _ = write!(out, "\x1b[{};{}H{}",
						self.number_node[i].0.1 + 1,
						self.number_node[i].0.0 + self.number_node[i].0.2 + 1,
						if self.answer[i] > self.answer[i + 1] { '-' } else if self.answer[i] < self.answer[i + 1] { '+' } else { '=' }
					);
				}
			}
		} else if self.game_result == 0 {
			for i in 0..data.difficulty as usize {
				let _ = write!(out, concat!(
						"\x1b[{0};{3}H",
						"\x1b[41;96;1m┌───┐\x1b[0m",
						"\x1b[{1};{3}H",
						"\x1b[41;96;1m│ {4} │\x1b[0m",
						"\x1b[{2};{3}H",
						"\x1b[41;96;1m└───┘\x1b[0m"),
						self.number_node[i].0.1,
						self.number_node[i].0.1 + 1,
						self.number_node[i].0.1 + 2,
						self.number_node[i].0.0,
						self.answer[i]);
			}
			let _ = write!(out, concat!(
					"\x1b[{};{}H",
					"╔═════════════════╗",
					"\x1b[19D\x1b[B",
					"║    \x1b[1mYou \x1b[31mLost!\x1b[0m    ║",
					"\x1b[19D\x1b[B",
					"║\x1b[17C║",
					"\x1b[19D\x1b[B",
					"║┌──────┐ ┌──────┐║",
					"\x1b[19D\x1b[B",
					"║│Change│ │ Back │║",
					"\x1b[19D\x1b[B",
					"║└──────┘ └──────┘║",
					"\x1b[19D\x1b[B",
					"║   ┌─────────┐   ║",
					"\x1b[19D\x1b[B",
					"║   │  Again  │   ║",
					"\x1b[19D\x1b[B",
					"║   └─────────┘   ║",
					"\x1b[19D\x1b[B",
					"╚═════════════════╝"
					),
					self.node[2].1, self.node[2].0 + 1);
		} else {
			for i in 0..data.difficulty as usize {
				let _ = write!(out, concat!(
						"\x1b[{0};{3}H",
						"\x1b[45;33;1m┌───┐\x1b[0m",
						"\x1b[{1};{3}H",
						"\x1b[45;33;1m│ {4} │\x1b[0m",
						"\x1b[{2};{3}H",
						"\x1b[45;33;1m└───┘\x1b[0m"),
						self.number_node[i].0.1,
						self.number_node[i].0.1 + 1,
						self.number_node[i].0.1 + 2,
						self.number_node[i].0.0,
						self.number_node[i].1);
			}
			let _ = write!(out, concat!(
					"\x1b[{};{}H",
					"╔═════════════════╗",
					"\x1b[19D\x1b[B",
					"║    \x1b[1mYou \x1b[33mWin!!\x1b[0m    ║",
					"\x1b[19D\x1b[B",
					"║\x1b[17C║",
					"\x1b[19D\x1b[B",
					"║┌──────┐ ┌──────┐║",
					"\x1b[19D\x1b[B",
					"║│Change│ │ Back │║",
					"\x1b[19D\x1b[B",
					"║└──────┘ └──────┘║",
					"\x1b[19D\x1b[B",
					"║   ┌─────────┐   ║",
					"\x1b[19D\x1b[B",
					"║   │  Again  │   ║",
					"\x1b[19D\x1b[B",
					"║   └─────────┘   ║",
					"\x1b[19D\x1b[B",
					"╚═════════════════╝"
					),
					self.node[2].1, self.node[2].0 + 1);
		}
	}

	fn switch_in(&mut self, _ctx: u8, data: &mut ContextData) {
		if data.first_time {
			self.restart(data);
			data.first_time = false;
		}
	}
}

struct HelpContext {
	node: [DataNode;3],
	page: u8
}

impl HelpContext {
	const PAGE_MAX : u8 = 4;
	fn new(w: DataSize, h: DataSize) -> Self {
		Self {
			node: [
				(0, h.saturating_sub(4), 11, 4),
				(w.saturating_sub(11), h.saturating_sub(4), 11, 4),
				((w/2).saturating_sub(6), h.saturating_sub(4), 12, 4)
			],
			page: 0
		}
	}
}

impl ContextTrait for HelpContext {
	fn input(&mut self, e: TermEvent, _data: &mut ContextData) -> ContextCommand {
		match e {
			TermEvent::Esc('\x1b') => ContextCommand::Switch(0),
			TermEvent::Click(x, y) => {
				if is_point_innode(x, y, self.node[0]) && self.page != 0 {
					self.page -= 1;
				} else if is_point_innode(x, y, self.node[1]) && self.page != Self::PAGE_MAX {
					self.page += 1;
				} else if is_point_innode(x, y, self.node[2]) {
					return ContextCommand::Switch(0);
				}
				ContextCommand::None
			},
			TermEvent::Resize(w, h) => {
				self.node[0].1 = h.saturating_sub(5);
				self.node[1].0 = w.saturating_sub(11);
				self.node[1].1 = h.saturating_sub(5);
				self.node[2].0 = (w/2).saturating_sub(6);
				self.node[2].1 = h.saturating_sub(4);
				ContextCommand::None
			}
			_ => ContextCommand::None
		}
	}

	fn render(&self, out: &mut StdoutLock<'_>, data: &ContextData) {
		let _ = data;
		let _ = write!(out, concat!(
				"\x1b[{}H",
				"{3}───────────┐\x1b[0m",
				"\x1b[12D\x1b[B",
				"\x1b[11C{3}│\x1b[0m",
				"\x1b[12D\x1b[B",
				"{3}  <- Prev  │\x1b[0m",
				"\x1b[12D\x1b[B",
				"\x1b[11C{3}│\x1b[0m",
				"\x1b[12D\x1b[B",
				"{3}───────────┘\x1b[0m",
				"\x1b[{};{}H",
				"{4}┌───────────\x1b[0m",
				"\x1b[11D\x1b[B",
				"{4}│\x1b[0m",
				"\x1b[D\x1b[B",
				"{4}│  Next ->  \x1b[0m",
				"\x1b[11D\x1b[B",
				"{4}│\x1b[0m",
				"\x1b[D\x1b[B",
				"{4}└───────────\x1b[0m",
				"\x1b[{5};{6}H",
				"┌──────────┐",
				"\x1b[12D\x1b[B",
				"│          │",
				"\x1b[12D\x1b[B",
				"│   Back   │",
				"\x1b[12D\x1b[B",
				"│          │",
				"\x1b[12D\x1b[B",
				"└──────────┘"
				),
				self.node[0].1 + 1,
				self.node[1].1 + 1, self.node[1].0,
				if self.page == 0 { "\x1b[38;2;128;128;128m" }
				else { "" },
				if self.page == Self::PAGE_MAX { "\x1b[38;2;128;128;128m" }
				else { "" },
				self.node[2].1, self.node[2].0 + 1);

		let _ = write!(out, "\x1b[H");
		let _ = out.write_all(&DB_HLINE_FULL[..3usize * data.width as usize]);
		let _ = write!(out, "\x1b[1;{}H                ", (data.width/2).saturating_sub(7));
		let _ = match self.page {
			0 => write!(out, concat!(
					"\x1b[1;{}HHelp\n\n",
					"Number Hangman is a game that combines\n",
					"\x1b[32;1mNumber Guessing\x1b[0m and \x1b[36;1mHangman\x1b[0m mechanic.\n\n",
					"Player \x1b[33;1mselect\x1b[0m and guess \x1b[36;1m1\x1b[0m digit at a time.\n",
					"Each \x1b[31;1mfailed\x1b[0m attempt will \x1b[32;1madvance\x1b[0m the hangman.\n\n ",
					" ┌┐\x1b[3D\x1b[B",
					" │════════╗\x1b[11D\x1b[B",
					" ││\x1b[7C│\x1b[11D\x1b[B",
					" ││\x1b[7C│\x1b[11D\x1b[B",
					" ││\x1b[6C╭┴╮\x1b[12D\x1b[B",
					" ││\x1b[6C╰┬╯\x1b[12D\x1b[B",
					" ││\x1b[6C╱│╲\x1b[12D\x1b[B",
					" ││\x1b[7C│\x1b[11D\x1b[B",
					" ││\x1b[6C╱ ╲\x1b[12D\x1b[B",
					" ││\x1b[3D\x1b[B",
					"║││║\x1b[4D\x1b[B",
					"╚══╝\x1b[4D\x1b[10A",
					"\x1b[25C",
					"┌───┐   \x1b[33;1m╔═══╗\x1b[0m   ┌───┐",
					"\x1b[21D\x1b[B",
					"│ \x1b[32;1m7\x1b[0m │   \x1b[33;1m║ \x1b[31;1m1\x1b[0m \x1b[33;1m║\x1b[0m   │   │",
					"\x1b[21D\x1b[B",
					"└───┘   \x1b[33;1m╚═══╝\x1b[0m   └───┘"
					), (data.width/2).saturating_sub(2) + 1),
			1 => write!(out, concat!(
					"\x1b[1;{}HDifficulty\n\n",
					"There are \x1b[33;1m3\x1b[0m selectable \x1b[31;1mdifficulty\x1b[0m.\n",
					"Each difficulty add \x1b[33;1mmore\x1b[0m digit to \x1b[32;1mguess\x1b[0m.\n\n",
					"\x1b[36;1mEasy Difficulty - 3 digits\x1b[0m\n",
					"┌───┐   ┌───┐   ┌───┐",
					"\n",
					"│   │   │   │   │   │",
					"\n",
					"└───┘   └───┘   └───┘\n\n",
					"\x1b[32;1mNormal Difficulty - 5 digits\x1b[0m\n",
					"┌───┐   ┌───┐   ┌───┐   ┌───┐   ┌───┐",
					"\n",
					"│   │   │   │   │   │   │   │   │   │",
					"\n",
					"└───┘   └───┘   └───┘   └───┘   └───┘\n\n",
					"\x1b[31;1mHard Difficulty - 7 digits\x1b[0m\n",
					"┌───┐   ┌───┐   ┌───┐   ┌───┐   ┌───┐   ┌───┐   ┌───┐",
					"\n",
					"│   │   │   │   │   │   │   │   │   │   │   │   │   │",
					"\n",
					"└───┘   └───┘   └───┘   └───┘   └───┘   └───┘   └───┘\n\n"
					), (data.width/2).saturating_sub(5) + 1),
			2 => write!(out, concat!(
					"\x1b[1;{}HHint\n\n",
					"There are \x1b[33;1m4\x1b[0m kinds of hints.\n",
					"These hints provide a way to deduce number using \x1b[32;1mlogic\x1b[0m.\n\n",
					"• \x1b[36;1mScratch Pad\x1b[0m\n",
					"This hint is a pad that \x1b[32;1mscratch\x1b[0m number if that number \x1b[35;1mdoesn't exist\x1b[0m in \x1b[33;1mall\x1b[0m digits.\n",
					"Has \x1b[31;1m3\x1b[0m slots that can run out and won't scratch further.\n\n",
					"• \x1b[36;1mHot/Cold & Odd/Even\x1b[0m\n",
					"These 2 hints tells wether a guess is \x1b[33;1mclose\x1b[0m to the answer, and wether the digit is \x1b[35;1modd or even\x1b[0m.\n",
					"These hints are \x1b[31;1mexclusive\x1b[0m to a single digit, and selecting different digits shows \x1b[32;1mits own hints\x1b[0m.\n",
					"\x1b[31;1mProgressively\x1b[0m shown on \x1b[32;1mattempt\x1b[0m. \x1b[35;1mHot/Cold\x1b[0m after 1st attempt and \x1b[35;1mOdd/Even\x1b[0m after 2nd.\n\n",
					"• \x1b[36;1mRelation\x1b[0m\n",
					"This hint shows relationship between a \x1b[33;1msolved\x1b[0m digit with its neighboring \x1b[35;1munsolved\x1b[0m digits.\n",
					"From the \x1b[33;1msolved\x1b[0m digit, '+' means the neighboring \x1b[35;1munsolved\x1b[0m digit is \x1b[32;1mbigger\x1b[0m while '-' means \x1b[32;1msmaller\x1b[0m and '=' means \x1b[32;1mequal\x1b[0m."
					), (data.width/2).saturating_sub(2) + 1),
			3 => write!(out, concat!(
					"\x1b[1;{}HGameplay\n\n",
					"Game start with digit count \x1b[35;1maccording\x1b[0m to difficulty selected.\n",
					"The \x1b[33;1mgoal\x1b[0m is to \x1b[33;1mguess\x1b[0m all digits before the hangman \x1b[31;1mcompleted\x1b[0m.\n\n",
					"Player \x1b[33;1mselect\x1b[0m a digit slot then \x1b[32;1minput\x1b[0m a number to guess that digit.\n",
					"\x1b[33;1m╔═══╗   ╔═══╗\n",
					"║   ║   ║ \x1b[32;1m1\x1b[0m \x1b[33;1m║\n",
					"╚═══╝   ╚═══╝\x1b[0m\n",
					"When player choose the correct number, it will be colored \x1b[33;1myellow\x1b[0m, and \x1b[31;1mred\x1b[0m when wrong.\n\n",
					"For each \x1b[31;1mfailed\x1b[0m attempt, the hangman \x1b[35;1madvance\x1b[0m once.\n",
					"There are \x1b[33;1m10\x1b[0m attempts available.\n\n",
					"The game \x1b[31;1mend\x1b[0m when the player either manage to guess all number (\x1b[33;1mWin\x1b[0m), or the hangman completed (\x1b[31;1mLost\x1b[0m)."
					), (data.width/2).saturating_sub(4) + 1),
			_ => write!(out, concat!(
					"\x1b[1;{}HGame End\n\n",
					"Game ends when player manage to \x1b[33;1mguess all number\x1b[0m, or when the \x1b[31;1mhangman completed\x1b[0m.\n\n",
					"When game ends, a \x1b[32;1.popup menu\x1b[0m showed up, telling wether player has \x1b[33;1mWon\x1b[0m or \x1b[31;1mlost\x1b[0m with 3 selectable menu.\n",
					"╔═════════════════╗",
					"\x1b[19D\x1b[B",
					"║    \x1b[1mYou \x1b[33mWin!!\x1b[0m    ║",
					"\x1b[19D\x1b[B",
					"║\x1b[17C║",
					"\x1b[19D\x1b[B",
					"║┌──────┐ ┌──────┐║",
					"\x1b[19D\x1b[B",
					"║│Change│ │ Back │║",
					"\x1b[19D\x1b[B",
					"║└──────┘ └──────┘║",
					"\x1b[19D\x1b[B",
					"║   ┌─────────┐   ║",
					"\x1b[19D\x1b[B",
					"║   │  Again  │   ║",
					"\x1b[19D\x1b[B",
					"║   └─────────┘   ║",
					"\x1b[19D\x1b[B",
					"╚═════════════════╝",
					"\x1b[9A     ",
					"╔═════════════════╗",
					"\x1b[19D\x1b[B",
					"║    \x1b[1mYou \x1b[31mLost!\x1b[0m    ║",
					"\x1b[19D\x1b[B",
					"║\x1b[17C║",
					"\x1b[19D\x1b[B",
					"║┌──────┐ ┌──────┐║",
					"\x1b[19D\x1b[B",
					"║│Change│ │ Back │║",
					"\x1b[19D\x1b[B",
					"║└──────┘ └──────┘║",
					"\x1b[19D\x1b[B",
					"║   ┌─────────┐   ║",
					"\x1b[19D\x1b[B",
					"║   │  Again  │   ║",
					"\x1b[19D\x1b[B",
					"║   └─────────┘   ║",
					"\x1b[19D\x1b[B",
					"╚═════════════════╝\n\n",
					"The \x1b[32;1m'Change'\x1b[0m button prompt back to \x1b[36;1mdifficulty selection\x1b[0m.\n",
					"The \x1b[32;1m'Back'\x1b[0m button prompt back to \x1b[36;1mtitle\x1b[0m.\n",
					"The \x1b[32;1m'Again'\x1b[0m button \x1b[36;1mrestart\x1b[0m the game."
					), (data.width/2).saturating_sub(4) + 1),
		};
	}

	fn switch_in(&mut self, _ctx : u8, _data: &mut ContextData) {
		self.page = 0;
	}
}

enum Context {
	Title(TitleContext),
	Difficulty(DifficultyContext),
	Game(GameContext),
	Help(HelpContext)
}

macro_rules! ctx_trait_fn {
	(
		[$($v:ident),* $(,)?],
		$f:ident, $( $mut:ident )? ($($slf:tt)+) | $rt:ty | $($arg:ident : $argty:ty),* $(,)?
	) => {
		fn $f(&$( $mut )? $($slf)+, $($arg : $argty,)*) -> $rt {
			ctx_trait_match!(
				[$($v,)*],
				($($slf)+), x, x.$f($($arg,)*)
			)
		}
	}
}

macro_rules! ctx_trait_match {
	(
		[$($v:ident),* $(,)?], 
		($($slf:tt)+), $x:ident, $call:expr
	) => {
		match $($slf)+ {
			$(
				Self::$v($x) => $call,
			)*
		}
	}
}

impl ContextTrait for Context {	
	ctx_trait_fn!(
		[Title, Difficulty, Game, Help],
		input, mut (self) | ContextCommand | e: TermEvent, data: &mut ContextData
	);

	ctx_trait_fn!(
		[Title, Difficulty, Game, Help],
		render, (self) | () | out: &mut StdoutLock<'_>, data: &ContextData
	);

	ctx_trait_fn!(
		[Title, Difficulty, Game, Help],
		switch_in, mut (self) | () | ctx: u8, data: &mut ContextData
	);

	ctx_trait_fn!(
		[Title, Difficulty, Game, Help],
		switch_out, mut (self) | () | ctx: u8, data: &mut ContextData
	);
}

fn main() {
	let mut setup = rft_term::TerminalSetup::new(&[rft_term::TermConfig::NoCursor, rft_term::TermConfig::EnableMouse]).expect("Unable to set up terminal");

	let mut data = {
		let (width, height) = setup.getSize();
		ContextData {
			width, height,
			difficulty: 0,
			first_time: true
		}
	};
	let mut contexts = [
		Context::Title(TitleContext::new(data.width, data.height)),
		Context::Difficulty(DifficultyContext::new(data.width, data.height)),
		Context::Game(GameContext::new(data.width, data.height)),
		Context::Help(HelpContext::new(data.width, data.height))
	];
	let mut ctx = 0u8;

	{
		let mut stdout_handle = io::stdout().lock();
		let mut stdin_handle = io::stdin().lock();

		contexts[ctx as usize].render(&mut stdout_handle, &data);
		loop {
			let _ = stdout_handle.flush();
			let _ = stdout_handle.write_all(b"\x1b[H\x1b[J");
			match contexts[ctx as usize].input({
				let e = setup.pollInput(&mut stdin_handle, -1);
				if let TermEvent::Resize(w, h) = e {
					data.width = w;
					data.height = h;
				}
				e
			}, &mut data) {
				ContextCommand::Switch(x) => {
					contexts[ctx as usize].switch_out(x, &mut data);
					contexts[x as usize].switch_in(ctx, &mut data);
					let _ = contexts[x as usize].input(TermEvent::Resize(data.width, data.height), &mut data);
					ctx = x;
				},
				ContextCommand::Exit => break,
				ContextCommand::None => {}
			}
			contexts[ctx as usize].render(&mut stdout_handle, &data);
		}
	}
}
