pub fn memset<T>(ptr: *mut T, value: u8, num: usize) {
    let mut ptr = ptr as *mut u8;
    for _ in 0..num {
        unsafe {
            *ptr = value;
            ptr = ptr.add(1);
        }
    }
}
