#[cfg(test)]
use std::sync::atomic::{AtomicU8, Ordering};
use std::{ffi::c_void, mem::size_of};

use windows::Win32::System::{
    Diagnostics::Debug::FlushInstructionCache,
    Memory::{
        MEM_COMMIT, MEM_IMAGE, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READWRITE, PAGE_GUARD,
        PAGE_NOACCESS, PAGE_PROTECTION_FLAGS, VirtualProtect, VirtualQuery,
    },
    Threading::GetCurrentProcess,
};

pub const OLD_BUDGET: u32 = 128 * 1024;
#[path = "clip_index.rs"]
mod clip_index;
pub const NEW_BUDGET: u32 = 1024 * 1024;
pub const PATCH_RVA: usize = 0xE83E1F;
const WINDOW_RVA: usize = 0xE83E13;
const TEXT_RVA: usize = 0x1000;
const TEXT_LENGTH: usize = 0x29A4800;
const IMAGE_LENGTH: usize = 0x5E0DA00;
const WORLD_RVA: usize = 0x3D69FF8;
const EDITS: [clip_index::Edit; 5] = [
    clip_index::Edit {
        rva: PATCH_RVA,
        original: &[2],
        replacement: &[0x10],
    },
    clip_index::EDITS[0],
    clip_index::EDITS[1],
    clip_index::EDITS[2],
    clip_index::EDITS[3],
];
const WINDOW: &[u8] = &[
    0x44, 0x89, 0x6c, 0x24, 0x30, 0x48, 0xc7, 0x44, 0x24, 0x28, 0, 0, 2, 0, 0x48, 0x89, 0x44, 0x24,
    0x20, 0x44, 0x8b, 0x4d, 0x9b, 0x41, 0xb8, 0, 0, 1, 0, 0x48, 0x8b, 0xd6, 0x49, 0x8b, 0xcc, 0xe8,
    0x25, 0x49, 0x05, 0x01,
];
const GUARDS: &[(usize, &[u8])] = &[
    (0x1461836, clip_index::TEARDOWN),
    (0x1462320, &[0x66, 0x89, 0x91, 0xc6, 0, 0, 0, 0xc3]),
    (0x1461a90, &[0x0f, 0xb7, 0x81, 0xc6, 0, 0, 0, 0xc3]),
    (
        0x1461836,
        &[
            0x0f, 0xb7, 0x83, 0xc6, 0, 0, 0, 0x66, 0x83, 0xf8, 0xfe, 0x75, 0x1a,
        ],
    ),
    (
        0x1461851,
        &[
            0x83, 0xc8, 0xff, 0x66, 0x89, 0x83, 0xc6, 0, 0, 0, 0xeb, 0x1e, 0x66, 0x83, 0xf8, 0xff,
            0x74, 0x18,
        ],
    ),
    (0x1460921, &[0x49, 0x8b, 6, 0x48, 0x8b, 0x88, 0xa8, 0, 0, 0]),
    (0x1460945, &[0xe8, 0x96, 0xd6, 0xfb, 0xff]),
    (
        0x1460a2c,
        &[
            0xb8, 0xfe, 0xff, 0xff, 0xff, 0x48, 0x8d, 0x4c, 0x24, 0x50, 0x4c, 0x8b, 0xc7, 0x66,
            0x89, 0x87, 0xc6, 0, 0, 0,
        ],
    ),
    (0x151527e, &[0x75, 0x45, 0xe8, 0x5b, 0xd7, 0x16, 0]),
    (
        0x15351db,
        &[
            0x48, 0x63, 0xc8, 0x48, 0x8d, 0x55, 0x7f, 0x48, 0x89, 0x4d, 0x7f,
        ],
    ),
    (
        0x15350ae,
        &[
            0x0f, 0xb6, 0x41, 0x53, 0x83, 0xf8, 4, 0x0f, 0x84, 5, 1, 0, 0,
        ],
    ),
    (WINDOW_RVA, WINDOW),
    (
        0x1ED87AB,
        &[
            0x48, 0x8b, 0x84, 0x24, 0xa8, 0, 0, 0, 0x48, 0x89, 0x46, 0x20,
        ],
    ),
    (
        0x1F3C57F,
        &[0x48, 0x8b, 0x4b, 0x20, 0xe8, 0x18, 0x06, 0xf8, 0xff],
    ),
    (
        0x1EBC8F2,
        &[
            0x48, 0x8b, 0x73, 0x10, 0x48, 0x85, 0xf6, 0x75, 0x06, 0x8b, 0x35, 0xf7, 0x7c, 0xc8, 1,
        ],
    ),
    (
        0x1EBC91F,
        &[
            0xe8, 0x5c, 0x24, 0xf6, 0xfe, 0x4c, 0x8b, 0, 0x48, 0x8b, 0xd6, 0x48, 0x8b, 0xc8, 0x41,
            0xff, 0x50, 0x48,
        ],
    ),
    (
        0x1EBCAAB,
        &[
            0x4c, 0x8d, 0x46, 0xc8, 0x48, 0x8d, 0x53, 0x38, 0x48, 0x8b, 0xcf, 0xe8, 0x05, 0x89,
            0x07, 0,
        ],
    ),
    (
        0x1F35530,
        &[
            0x49, 0x8d, 0x40, 0xff, 0x4c, 0x8b, 0xda, 0x4c, 0x8b, 0xc9, 0x49, 0x85, 0xc0, 0x74, 3,
            0x33, 0xc0, 0xc3, 0x4c, 0x8b, 0x51, 0x18, 0xb8, 8, 0, 0, 0, 0x49, 0x8b, 0xd2, 0x49,
            0x2b, 0xd3, 0x4c, 0x3b, 0xc0, 0x49, 0x0f, 0x43, 0xc0, 0x48, 0xff, 0xc8, 0x48, 0x23,
            0xc2, 0x48, 0x2b, 0xd0, 0x33, 0xc0, 0x48, 0x8d, 0x4a, 0xf0, 0x49, 0x39, 0x49, 0x10,
            0x77, 0x12, 0x4c, 0x89, 0x11, 0x48, 0x8b, 0xc2, 0x48, 0xc7, 0x41, 8, 1, 0, 0, 0, 0x49,
            0x89, 0x49, 0x18, 0xc3,
        ],
    ),
];

struct Profile {
    timestamp: u32,
    image_length: usize,
    text_length: usize,
    world_rva: usize,
    window_rva: usize,
    window: &'static [u8],
    guards: &'static [(usize, &'static [u8])],
    edits: &'static [clip_index::Edit],
}
const LEGACY: Profile = Profile {
    timestamp: 0x6a96b418,
    image_length: IMAGE_LENGTH,
    text_length: TEXT_LENGTH,
    world_rva: WORLD_RVA,
    window_rva: WINDOW_RVA,
    window: WINDOW,
    guards: GUARDS,
    edits: &EDITS,
};
#[path = "profile272.rs"]
mod profile272;
fn select_profile(timestamp: u32) -> Result<&'static Profile, &'static str> {
    match timestamp {
        0x6a96b418 => Ok(&LEGACY),
        0x6aabf7e7 => Ok(&profile272::PROFILE),
        _ => Err("unsupported PE build; expected audited ER 2.7.1.0 or 2.7.2.0"),
    }
}

fn u16_at(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes.get(offset..offset + 2)?.try_into().ok()?,
    ))
}

fn u32_at(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn validate<'a>(read: impl Fn(usize, usize) -> Option<&'a [u8]>) -> Result<(), &'static str> {
    let header = read(0, 4096).ok_or("unreadable PE header")?;
    if header.get(..2) != Some(b"MZ") {
        return Err("invalid DOS header");
    }
    let nt = u32_at(header, 0x3c).ok_or("invalid PE offset")? as usize;
    if nt > 1024 || header.get(nt..nt + 4) != Some(b"PE\0\0") {
        return Err("invalid PE signature");
    }
    let profile = select_profile(u32_at(header, nt + 8).ok_or("invalid timestamp")?)?;
    if u16_at(header, nt + 4) != Some(0x8664)
        || u16_at(header, nt + 6) != Some(11)
        || u32_at(header, nt + 8) != Some(profile.timestamp)
        || u16_at(header, nt + 20) != Some(240)
        || u16_at(header, nt + 24) != Some(0x20b)
        || u32_at(header, nt + 24 + 56) != Some(profile.image_length as u32)
    {
        return Err("unsupported PE build; expected audited ER 2.7.1.0");
    }
    let section = nt + 24 + 240;
    if header.get(section..section + 8) != Some(b".text\0\0\0")
        || u32_at(header, section + 8) != Some(profile.text_length as u32)
        || u32_at(header, section + 12) != Some(TEXT_RVA as u32)
    {
        return Err("unexpected executable section layout");
    }
    for &(rva, expected) in profile.guards {
        if read(rva, expected.len()) != Some(expected) {
            return Err("instruction guard mismatch or conflicting patch; no writes");
        }
    }
    for edit in profile.edits {
        if edit.original.len() != edit.replacement.len()
            || edit.original.is_empty()
            || edit.original.len() > 4096
            || edit.rva + edit.original.len() > profile.image_length
            || read(edit.rva, edit.original.len()) != Some(edit.original)
        {
            return Err("clip-index instruction guard mismatch or conflicting patch; no writes");
        }
    }
    if read(profile.world_rva, 8) != Some(&[0u8; 8]) {
        return Err(
            "game world already initialized or unreadable; startup loading required; no writes",
        );
    }
    let text = read(TEXT_RVA, profile.text_length).ok_or("unreadable executable section")?;
    let mut matches = text
        .windows(profile.window.len())
        .enumerate()
        .filter(|(_, b)| *b == profile.window);
    if matches.next().map(|(offset, _)| offset + TEXT_RVA) != Some(profile.window_rva)
        || matches.next().is_some()
    {
        return Err("constructor signature missing or ambiguous");
    }
    Ok(())
}

// Only committed, readable regions belonging to the original mapped PE are
// accepted. No external process or heap pointers are followed.
unsafe fn image_bytes<'a>(base: usize, rva: usize, length: usize) -> Option<&'a [u8]> {
    if rva.checked_add(length)? > IMAGE_LENGTH {
        return None;
    }
    let start = base.checked_add(rva)?;
    let end = start.checked_add(length)?;
    let mut cursor = start;
    while cursor < end {
        let mut info = MEMORY_BASIC_INFORMATION::default();
        if unsafe {
            VirtualQuery(
                Some(cursor as *const c_void),
                &mut info,
                size_of::<MEMORY_BASIC_INFORMATION>(),
            )
        } == 0
            || info.State != MEM_COMMIT
            || info.Type != MEM_IMAGE
            || info.AllocationBase as usize != base
            || info.Protect.0 & (PAGE_GUARD.0 | PAGE_NOACCESS.0) != 0
            || info.Protect.0 & 0xee == 0
        {
            return None;
        }
        let next = (info.BaseAddress as usize).checked_add(info.RegionSize)?;
        if next <= cursor {
            return None;
        }
        cursor = next;
    }
    Some(unsafe { std::slice::from_raw_parts(start as *const u8, length) })
}

pub unsafe fn apply(base: usize) -> Result<(), &'static str> {
    validate(|rva, length| unsafe { image_bytes(base, rva, length) })?;
    let header = unsafe { image_bytes(base, 0, 4096) }.ok_or("unreadable PE header")?;
    let nt = u32_at(header, 0x3c).ok_or("invalid PE offset")? as usize;
    let profile = select_profile(u32_at(header, nt + 8).ok_or("invalid PE timestamp")?)?;
    unsafe { write_edits_for_profile(base, profile, || true) }
}

// Startup-only: callers must ensure no thread can execute these windows yet.
// The callback permits deterministic API failure tests without game injection.
unsafe fn write_edits_for_profile(
    base: usize,
    profile: &Profile,
    mut before_api: impl FnMut() -> bool,
) -> Result<(), &'static str> {
    let mut pages = [0usize; 10];
    let mut protections = [PAGE_PROTECTION_FLAGS::default(); 10];
    let mut count = 0;
    for edit in profile.edits {
        let first = (base + edit.rva) & !4095;
        let last = (base + edit.rva + edit.original.len() - 1) & !4095;
        for page in (first..=last).step_by(4096) {
            if !pages[..count].contains(&page) {
                pages[count] = page;
                count += 1;
            }
        }
    }
    let protect = |page: usize, flags, old: &mut PAGE_PROTECTION_FLAGS| unsafe {
        VirtualProtect(page as *const c_void, 4096, flags, old).is_ok()
    };
    let flush = || unsafe {
        FlushInstructionCache(
            GetCurrentProcess(),
            Some(base as *const c_void),
            profile.image_length,
        )
        .is_ok()
    };
    for i in 0..count {
        if !before_api() || !protect(pages[i], PAGE_EXECUTE_READWRITE, &mut protections[i]) {
            for j in 0..i {
                let _ = protect(
                    pages[j],
                    protections[j],
                    &mut PAGE_PROTECTION_FLAGS::default(),
                );
            }
            return Err(
                "page preparation failed; no instruction writes; protection recovery attempted",
            );
        }
    }
    let originals_match = profile.edits.iter().all(|edit| unsafe {
        std::slice::from_raw_parts((base + edit.rva) as *const u8, edit.original.len())
            == edit.original
    });
    if originals_match {
        for edit in profile.edits {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    edit.replacement.as_ptr(),
                    (base + edit.rva) as *mut u8,
                    edit.replacement.len(),
                )
            };
        }
    }
    let mut success = originals_match && before_api() && flush();
    for i in 0..count {
        if !before_api()
            || !protect(
                pages[i],
                protections[i],
                &mut PAGE_PROTECTION_FLAGS::default(),
            )
        {
            success = false;
        }
    }
    if success {
        return Ok(());
    }
    // Some pages may already be RX. Reopen before rollback, never store to a
    // page whose protection could not be acquired, and do not undo foreign bytes.
    let mut writable = [false; 10];
    for i in 0..count {
        writable[i] = protect(
            pages[i],
            PAGE_EXECUTE_READWRITE,
            &mut PAGE_PROTECTION_FLAGS::default(),
        );
    }
    if originals_match {
        for edit in profile.edits {
            let first = (base + edit.rva) & !4095;
            let last = (base + edit.rva + edit.original.len() - 1) & !4095;
            let all_writable = (first..=last).step_by(4096).all(|page| {
                pages[..count]
                    .iter()
                    .position(|p| *p == page)
                    .is_some_and(|i| writable[i])
            });
            let current = unsafe {
                std::slice::from_raw_parts((base + edit.rva) as *const u8, edit.replacement.len())
            };
            if all_writable && current == edit.replacement {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        edit.original.as_ptr(),
                        (base + edit.rva) as *mut u8,
                        edit.original.len(),
                    )
                };
            }
        }
    }
    let _ = flush();
    for i in 0..count {
        let _ = protect(
            pages[i],
            protections[i],
            &mut PAGE_PROTECTION_FLAGS::default(),
        );
    }
    Err("patch transaction failed or bytes changed concurrently; rollback attempted; restart game")
}

#[cfg(test)]
unsafe fn write_budget_byte(address: *mut u8) -> Result<(), &'static str> {
    let mut original_protection = PAGE_PROTECTION_FLAGS::default();
    unsafe {
        VirtualProtect(
            address.cast(),
            1,
            PAGE_EXECUTE_READWRITE,
            &mut original_protection,
        )
    }
    .map_err(|_| "VirtualProtect failed; budget unchanged")?;
    // A byte-sized update avoids torn multi-byte immediates. Threads that have
    // already fetched the old instruction still need a clean startup/restart.
    let byte = unsafe { &*address.cast::<AtomicU8>() };
    let changed = byte
        .compare_exchange(2, 0x10, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok();
    let flushed = !changed
        || unsafe { FlushInstructionCache(GetCurrentProcess(), Some(address.cast()), 1) }.is_ok();
    let mut ignored = PAGE_PROTECTION_FLAGS::default();
    if !flushed {
        let _ = byte.compare_exchange(0x10, 2, Ordering::SeqCst, Ordering::SeqCst);
        let _ = unsafe { FlushInstructionCache(GetCurrentProcess(), Some(address.cast()), 1) };
    }
    let restored =
        unsafe { VirtualProtect(address.cast(), 1, original_protection, &mut ignored) }.is_ok();
    if !restored {
        // The page is still writable after a failed restore. Try to undo our
        // byte and restore again; never claim this path as successful.
        if changed {
            let _ = byte.compare_exchange(0x10, 2, Ordering::SeqCst, Ordering::SeqCst);
        }
        let _ = unsafe { FlushInstructionCache(GetCurrentProcess(), Some(address.cast()), 1) };
        let _ = unsafe { VirtualProtect(address.cast(), 1, original_protection, &mut ignored) };
        return Err("protection restore failed; rollback attempted; restart game");
    }
    if !flushed {
        return Err("instruction cache flush failed; rollback attempted; restart game");
    }
    if !changed {
        return Err("budget changed concurrently; no budget write performed");
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
