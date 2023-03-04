use crate::atags::raw;

pub use crate::atags::raw::{Core, Mem};

/// An ATAG.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Atag {
    Core(raw::Core),
    Mem(raw::Mem),
    Cmd(&'static str),
    Unknown(u32),
    None,
}

impl Atag {
    /// Returns `Some` if this is a `Core` ATAG. Otherwise returns `None`.
    pub fn core(self) -> Option<Core> {
        if let Atag::Core(core) = self {
            Some(core)
        } else {
            None
        }
    }

    /// Returns `Some` if this is a `Mem` ATAG. Otherwise returns `None`.
    pub fn mem(self) -> Option<Mem> {
        if let Atag::Mem(mem) = self {
            Some(mem)
        } else {
            None
        }
    }

    /// Returns `Some` with the command line string if this is a `Cmd` ATAG.
    /// Otherwise returns `None`.
    pub fn cmd(self) -> Option<&'static str> {
        if let Atag::Cmd(s) = self {
            Some(s)
        } else {
            None
        }
    }
}

impl From<&'static raw::Atag> for Atag {
    fn from(atag: &'static raw::Atag) -> Self {
        unsafe {
            match (atag.tag, &atag.kind) {
                (raw::Atag::CORE, &raw::Kind { core }) => Atag::Core(core),
                (raw::Atag::MEM, &raw::Kind { mem }) => Atag::Mem(mem),
                (raw::Atag::CMDLINE, &raw::Kind { ref cmd }) => cmd.into(),
                (raw::Atag::NONE, _) => Atag::None,
                (id, _) => Atag::Unknown(id),
            }
        }
    }
}

impl From<&raw::Cmd> for Atag {
    fn from(cmd: &raw::Cmd) -> Self {
        // We're going to read a C style string, one byte at a time, starting at this address.
        let start_addr = cmd as *const raw::Cmd as *const u8;

        let str_as_bytes = unsafe {
            // We can't just use the size information from the tag header, because it must maintain
            // a 4 byte alignment. The actual string may terminate after 5 bytes, so we need to
            // just search for the null terminator.
            let mut cur_addr = start_addr;
            while *cur_addr != b'\0' {
                cur_addr = cur_addr.add(1);
            }

            let len = cur_addr.offset_from(start_addr) as usize; // we know this is positive
            core::slice::from_raw_parts(start_addr, len)
        };

        let s =
            core::str::from_utf8(str_as_bytes).expect("Failed to parse ATAG command line to UTF-8");
        Atag::Cmd(s)
    }
}
