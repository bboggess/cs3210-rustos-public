/// A raw `ATAG` as laid out in memory.
#[repr(C)]
pub struct Atag {
    pub dwords: u32,
    pub tag: u32,
    pub kind: Kind,
}

impl Atag {
    pub const NONE: u32 = 0x00000000;
    pub const CORE: u32 = 0x54410001;
    pub const MEM: u32 = 0x54410002;
    pub const VIDEOTEXT: u32 = 0x54410003;
    pub const RAMDISK: u32 = 0x54410004;
    pub const INITRD2: u32 = 0x54420005;
    pub const SERIAL: u32 = 0x54410006;
    pub const REVISION: u32 = 0x54410007;
    pub const VIDEOLFB: u32 = 0x54410008;
    pub const CMDLINE: u32 = 0x54410009;

    pub fn next(&self) -> Option<&Atag> {
        // We're going to do unsafe things with addresses later, so
        // let's be extra pedantic and make sure we defintely have a next tag
        if self.tag == Atag::NONE {
            return None;
        }

        let cur_start_addr = self as *const Atag as *const u32;
        let cur_tag_size = self.dwords as usize; // counts number of 32 bit dwords in the current tag

        unsafe {
            let next_tag = cur_start_addr.add(cur_tag_size) as *const Atag;
            Some(&*next_tag)
        }
    }
}

/// The possible variant of an ATAG.
#[repr(C)]
pub union Kind {
    pub core: Core,
    pub mem: Mem,
    pub cmd: Cmd,
}

/// A `CORE` ATAG.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Core {
    pub flags: u32,
    pub page_size: u32,
    pub root_dev: u32,
}

/// A `MEM` ATAG.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Mem {
    pub size: u32,
    pub start: u32,
}

/// A `CMDLINE` ATAG.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Cmd {
    /// The first byte of the command line string.
    pub cmd: u8,
}
