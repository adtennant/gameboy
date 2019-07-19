use crate::{cartridge::Cartridge, rom::ROM, Console};
use std::ffi::CString;

#[no_mangle]
pub extern "C" fn gb_create() -> *mut Console {
    let gb = Console::new();
    Box::into_raw(Box::new(gb))
}

#[no_mangle]
pub unsafe extern "C" fn gb_destroy(gb: *mut Console) {
    assert!(!gb.is_null());

    drop(Box::from_raw(gb));
}

#[no_mangle]
pub unsafe extern "C" fn gb_load_rom(
    gb: *mut Console,
    path: *const std::os::raw::c_char,
    title: *mut std::os::raw::c_char,
) {
    assert!(!path.is_null());

    let path = std::ffi::CStr::from_ptr(path)
        .to_string_lossy()
        .into_owned();
    println!("Loading {:?}", path);

    let rom = ROM::from_file(path).unwrap();
    let rom_title = match CString::new(rom.title()) {
        Ok(title) => title,
        _ => panic!(),
    };

    let buf: &mut [std::os::raw::c_char] = std::slice::from_raw_parts_mut(title, 16);
    rom_title.into_bytes_with_nul().iter().enumerate().for_each(|(i, c)| {
        buf[i] = *c as i8;
    });

    let cartridge = Cartridge::from(rom);
    (&mut *gb).insert_cartridge(cartridge);
}

#[no_mangle]
pub unsafe extern "C" fn gb_run_frame(gb: *mut Console) {
    assert!(!gb.is_null());

    let start = std::time::Instant::now();

    (&mut *gb).run_frame();

    let end = std::time::Instant::now();
    println!("{:?}", end - start);
}

#[no_mangle]
pub unsafe extern "C" fn gb_get_frame_buffer(gb: *mut Console, buf: *mut std::os::raw::c_uchar) {
    assert!(!gb.is_null());
    assert!(!buf.is_null());

    let framebuffer = (&mut *gb).video.framebuffer();
    let framebuffer: Vec<_> = framebuffer.iter().map(|x| *x as u8).collect();

    let buf: &mut [std::os::raw::c_uchar] = std::slice::from_raw_parts_mut(buf, 160 * 144);
    buf.copy_from_slice(&framebuffer);
}
