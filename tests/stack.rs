use baffa::{WriteBuf, StaticBuffer};
use core::{mem, slice};

#[test]
fn test_stack_buffer() {
    let num = u64::max_value();

    let bytes = unsafe {
        slice::from_raw_parts(&num as *const _ as *const u8, mem::size_of::<u64>())
    };

    let mut buffer = StaticBuffer::<u64>::new();
    assert_eq!(buffer.write_slice(bytes), 8);
    assert_eq!(buffer.as_slice(), bytes);
    assert_eq!(buffer.write_slice(bytes), 0);
    assert_eq!(buffer.as_slice(), bytes);

    let mut buffer = StaticBuffer::<u32>::new();
    assert_eq!(buffer.write_slice(bytes), 4);

    let num = u32::max_value();
    let bytes = unsafe {
        slice::from_raw_parts(&num as *const _ as *const u8, mem::size_of::<u32>())
    };
    assert_eq!(buffer.as_slice(), bytes);
}
