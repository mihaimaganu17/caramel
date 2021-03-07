// #![no_std]

use core::convert::TryInto;

// Const for machine types
const IMAGE_FILE_MACHINE_I386: u16 = 0x014c;
const IMAGE_FILE_MACHINE_AMD64: u16 = 0x8664;

struct PeParser<'a> {
    /// Raw PE file
    bytes: &'a [u8],

    /// Number of sections
    nsections: usize,

    /// Offset into the raw PE file where the section headers are
    section_off: usize,

    /// Base of the image
    image_base: u64,
}

impl<'a> PeParser<'a> {
    fn parse(bytes: &'a [u8]) -> Option<Self> {
        let bytes: &[u8] = bytes.as_ref();

        // Check for an MZ header
        if bytes.get(0..2) != Some(b"MZ") { return None; }

        // Get PE offset -> e_lfanew
        let pe_offset: usize = u32::from_le_bytes(bytes.get(0x3c..0x40)?
                .try_into().ok()?) as usize;

        // Check for the PE signature -> overflow safe
        if bytes.get(pe_offset..pe_offset.checked_add(4)?) != Some(b"PE\0\0") {
            return None;
        }

        // Make sure the COFF header is within bounds of our input
        if pe_offset.checked_add(0x18)? > bytes.len() {
            return None;
        }

        // Determine the machine type
        let machine: u16 = u16::from_le_bytes(
            bytes[pe_offset + 4..pe_offset +6].try_into().ok()?);
        if machine != IMAGE_FILE_MACHINE_I386 &&
                machine != IMAGE_FILE_MACHINE_AMD64 {
            return None;
        }

        // Get number of sections
        let nsections: usize = u16::from_le_bytes(
            bytes[pe_offset + 6..pe_offset + 8].try_into().ok()?)
            .try_into().ok()?;

        // Get the size of the optional header
        let opt_header_size: usize = u16::from_le_bytes(
            bytes[pe_offset + 0x14..pe_offset + 0x16].try_into().ok()?)
            .try_into().ok()?;

        // Get the base for the program
        // Upcasts are always fine/safe, downcasts are not
        let image_base = if machine == IMAGE_FILE_MACHINE_I386 {
            u32::from_le_bytes(
                bytes.get(pe_offset + 0x34..pe_offset + 0x38)?
                .try_into().ok()?) as u64
        } else if machine == IMAGE_FILE_MACHINE_AMD64 {
            u64::from_le_bytes(
                bytes.get(pe_offset + 0x30..pe_offset + 0x38)?
                .try_into().ok()?)
        } else {
            unreachable!();
        };

        // Computer the size of all headers, including sections
        // and make sure everything is in bounds
        let header_size = pe_offset.checked_add(0x18)?
            .checked_add(opt_header_size)?
            .checked_add(nsections.checked_mul(0x28)?)?;
        if header_size > bytes.len() {
            return None;
        }


        Some(PeParser {
            bytes,
            image_base,
            nsections,
            section_off: pe_offset + 0x18 + opt_header_size,
        })
    }

    /// Call a closure with the section
    fn sections<F: FnMut(u64, u32, &[u8])>(&self, mut func: F) -> Option<()> {
        let bytes = self.bytes;

        for section in 0..self.nsections {
            // This arithmetic cannot overflow as we validated
            // the entire header size above
            let off = self.section_off + section * 0x28;

            let virt_size = u32::from_le_bytes(
                bytes[off + 0x8..off + 0xc].try_into().ok()?);
            let virt_addr = u32::from_le_bytes(
                bytes[off + 0xc..off + 0x10].try_into().ok()?);
            let raw_size = u32::from_le_bytes(
                bytes[off + 0x10..off + 0x14].try_into().ok()?);
            let raw_off: usize = u32::from_le_bytes(
                bytes[off + 0x14..off + 0x18].try_into().ok()?)
                .try_into().ok()?;

            // Truncate the raw size if it exceeds the section size
            let raw_size: usize = core::cmp::min(raw_size, virt_size)
                    .try_into().ok()?;

            // Invoke the closure
            func(
                self.image_base.checked_add(virt_addr as u64)?,
                virt_size,
                bytes.get(raw_off..raw_off.checked_add(raw_size)?)?);
        }

        Some(())
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use crate::*;

    #[test]
    fn it_works() {
        let pe = std::fs::read("bootloader.exe").unwrap();
        let pe = PeParser::parse(&pe).unwrap();
        pe.sections(|base, size, raw|{
            std::print!("{:#x} {:#x} {:02x?}\n", base, size, raw);
        });
    }
}
