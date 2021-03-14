use core::convert::TryInto;
use crate::realmode::{invoke_realmode, pxecall, RegisterState};

/// Convert a 16-bit seg:off pointer into a linear address
fn segoff_to_linear(seg: u16, off: u16) -> usize {
    ((seg as usize) << 4) + off as usize
}

pub fn download<P: AsRef<[u8]>>(filename: P) -> Option<()> {
    // Convert the filename to a slice of bytes
    let filename = filename.as_ref();

    // Invoke the PXE isntallation check with int 0x1a
    let mut regs = RegisterState::default();
    regs.eax = 0x5650;
    unsafe { invoke_realmode(0x1a, &mut regs); }

    // Check that the PXE API responded as expected
    if regs.eax != 0x564e || (regs.efl & 1) != 0 {
        return None;
    }

    // Get the linear address to the PXENV+ structure
    let pxenv = segoff_to_linear(regs.es, regs.ebx as u16);
    let pxenv = unsafe {
        core::slice::from_raw_parts(pxenv as *const u8, 0x2c)
    };

    // Extract the fields we need to validate the structure
    let signature = &pxenv[..6];
    let length = pxenv[0x8];
    let checksum = pxenv.iter().fold(0u8, |acc, &x| acc.wrapping_add(x));

    // Check the signature and length for sanity
    if signature != b"PXENV+" || length != 0x2c || checksum != 0 {
        return None;
    }

    // Get the pointer to the !PXE structure
    let off = u16::from_le_bytes(pxenv[0x28..0x2a].try_into().ok()?);
    let seg = u16::from_le_bytes(pxenv[0x2a..0x2c].try_into().ok()?);
    let pxe = segoff_to_linear(seg, off);
    let pxe = unsafe {
        core::slice::from_raw_parts(pxe as *const u8, 0x58)
    };

    // Extract the field we need to validate the !PXE structure
    let signature = &pxe[..4];
    let length = pxe[4];
    let checksum = pxe.iter().fold(0u8, |acc, &x| acc.wrapping_add(x));

    // Check for correctness
    if signature != b"!PXE" || length != 0x58 || checksum != 0 {
        return None;
    }

    // Get the 16-bit PXE API entry point
    let ep_off = u16::from_le_bytes(pxe[0x10..0x12].try_into().ok()?);
    let ep_seg = u16::from_le_bytes(pxe[0x12..0x14].try_into().ok()?);

    // According to the spec, CS must not be 0000h
    if ep_seg == 0 {
        return None;
    }

    // Determine the server IP from the cached information used during the
    // PXE boot process. We grab the DHCP ACK packet and extract the server IOP
    // field from it.
    let server_ip: [u8; 4] = {
        const PXE_GET_CACHED_INFO: u16 = 0x71;
        const PXENV_PACKET_TYPE_DHCP_ACK: u16 = 2;
        #[derive(Debug, Default)]
        #[repr(C)]
        struct GetCachedInfo {
            status: u16,
            packet_type: u16,
            buffer_size: u16,
            buffer_off: u16,
            buffer_seg: u16,
            buffer_limit: u16,
        }

        // Buffer to hold the DHCP ACK packer
        let mut pkt_buf = [0u8; 128];

        // Request the DHCP ACK packet
        let mut st = GetCachedInfo::default();
        st.packet_type = PXENV_PACKET_TYPE_DHCP_ACK;
        st.buffer_size = 128;
        st.buffer_seg = 0;
        st.buffer_off = &mut pkt_buf as *mut _ as u16;

        unsafe {
            pxecall(ep_seg, ep_off, PXE_GET_CACHED_INFO,
                0, &mut st as *mut _ as u16);
        }

        // Make sure this call was successful
        if st.status != 0 {
            return None;
        }

        // Extract the serve IP
        pkt_buf[0x14..0x18].try_into().ok()?

    }

    serial::print!("Server IP: {}.{}.{}.{}\n",
                server_ip[0], server_ip[1], server_ip[2], server_ip[3]);
    // Get the file size for the next stage
    {
        const PXE_TFTP_GET_FILE_SIZE: u16 = 0x71;

        #[derive(Default)]
        struct GetFileSize {
            status: u16,
            server_ip: [u8; 4],
            gateway_ip: [u8; 4],
            filename: [u8; 128],
            file_size: u32,
        }

        let mut st = GetFileSize::default();
        st.server_ip = server_ip;
        st.gateway_ip = [0; 4];

        // Check to see if we have enough room for the filename and null
        // terminator
        if filename.len() + 1 > st.filename.len() {
            return None;
        }

        // Copy in the filename
        st.filename.copy_from_slice(filename);

        unsafe {
            pxecall(ep_seg, 
        }

    }

    Some(())
}
