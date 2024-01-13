//! Access flags for segments.
//!
//! See Intel 3a, Section 3.4.5 "Segment Descriptors"
//! and Section 3.5 "System Descriptor Types"

/// Maximum segment size
pub const MAX_SEGMENT_SIZE: usize = 0xfffff;
/// No offset
pub const NO_OFFSET: usize = 0;

/// Returns the access flags for a null segment.
pub const NULL_SEGMENT: u8 = 0;

/// Returns the access flags for a kernel code segment.
pub const KERNEL_CODE_SEGMENT: u8 = PRESENT | TYPE | EXECUTABLE | READABLE_WRITABLE;

/// Returns the access flags for a kernel data segment.
pub const KERNEL_DATA_SEGMENT: u8 = PRESENT | TYPE | READABLE_WRITABLE;

/// Returns the access flags for a kernel stack segment.
pub const KERNEL_STACK_SEGMENT: u8 = PRESENT | TYPE | READABLE_WRITABLE | DIRECTION_CONFORMING;

/// Returns the access flags for a user code segment.
pub const USER_CODE_SEGMENT: u8 = PRESENT | DPL | TYPE | READABLE_WRITABLE | EXECUTABLE;

/// Returns the access flags for a user data segment.
pub const USER_DATA_SEGMENT: u8 = PRESENT | DPL | TYPE | READABLE_WRITABLE;

/// Returns the access flags for a user stack segment.
pub const USER_STACK_SEGMENT: u8 = PRESENT | DPL | TYPE | READABLE_WRITABLE | DIRECTION_CONFORMING;

/// Flags plus the high bits of the segment size. A u8 composed of 2 u4s.
pub const SEGMENT_FLAGS: u8 = GRANULARITY | DB_SIZE | SEGMENT_SIZE_HIGH;

/// Set for the limit scale to be 4KB blocks. If not set, the limit scale is bytes.
const GRANULARITY: u8 = 1 << 7;
/// Set for 32-bit protected mode. If not set, the segment is 16-bit.
const DB_SIZE: u8 = 1 << 6;
/// Set for 64 bit code segment. If not set, the segment is 32-bit.
#[allow(dead_code)]
const LONG_MODE: u8 = 1 << 5;
/// Reserved bit. Must be 0.
#[allow(dead_code)]
const RESERVED: u8 = 1 << 4;
/// High bits of the limit: u4. Rest is 0.
const SEGMENT_SIZE_HIGH: u8 = (MAX_SEGMENT_SIZE >> 16) as u8;

/// Set if the segment is present in memory.
const PRESENT: u8 = 1 << 7;
/// Set the CPU privilege level. 0 for kernel, 3 for user.
const DPL: u8 = 1 << 6 | 1 << 5;
/// Set the type of segment. 0 for system, 1 for code/data.
const TYPE: u8 = 1 << 4;
/// Set if the segment is executable.
const EXECUTABLE: u8 = 1 << 3;
/// If segment is code, 1 for lower privilege levels, 0 for same privilege level
/// If segment is data, 1 for grows up, 0 for grow down
const DIRECTION_CONFORMING: u8 = 1 << 2;
/// If segment is code, 1 for readable. If segment is data, 1 for writable.
const READABLE_WRITABLE: u8 = 1 << 1;
/// If segment is accessed, CPU will set this flag.
#[allow(dead_code)]
const ACCESSED: u8 = 1 << 0;
