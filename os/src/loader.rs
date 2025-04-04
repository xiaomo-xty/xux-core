//! Loading user applications into memory

pub fn get_num_app() -> usize {
    extern "C" {
        fn _num_app();
    }
    unsafe {
        let num_app_ptr = _num_app as usize as *const usize;
        num_app_ptr.read_volatile()
    }
}

pub fn get_app_data(app_id: usize) -> &'static [u8] {
    extern "C" {
        fn _num_app();
    }
    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();
    assert!(app_id < num_app);
    let app_start = unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) };

    unsafe {
        core::slice::from_raw_parts(
            app_start[app_id] as *const u8,
            app_start[app_id + 1] - app_start[app_id],
        )
    }
}
