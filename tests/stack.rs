use baffa::{WriteBuf, WriteBufExt, StaticBuffer, ReadBuf, ReadBufExt};
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

    let mut buffer = StaticBuffer::<u8>::new();
    assert_eq!(buffer.write_value(&241u8), 1);
    assert_eq!(buffer.as_slice(), [241]);

}

#[test]
fn test_ring_buffer() {
    let num = u64::max_value();

    let bytes = unsafe {
        slice::from_raw_parts(&num as *const _ as *const u8, mem::size_of::<u64>())
    };

    let mut buffer = StaticBuffer::<u64>::new().into_circular();
    assert_eq!(buffer.available(), 0);
    assert_eq!(buffer.write_slice(bytes), 8);
    assert_eq!(buffer.available(), 8);
    assert_eq!(buffer.write_slice(bytes), 8);
    assert_eq!(buffer.available(), 8);
    assert_eq!(buffer.write_value(&0u32), 4);
    assert_eq!(buffer.available(), 8);

    let mut res = mem::MaybeUninit::<u32>::new(0);
    assert_eq!(buffer.read_value(&mut res), 4);
    assert_eq!(unsafe {
        res.assume_init()
    }, u32::max_value());

    assert_eq!(buffer.available(), 4);

    let mut res = mem::MaybeUninit::<u32>::new(1);
    assert_eq!(buffer.read_value(&mut res), 4);
    assert_eq!(unsafe {
        res.assume_init()
    }, 0);

    assert_eq!(buffer.available(), 0);
    assert_eq!(buffer.read_value(&mut res), 0);
    assert_eq!(unsafe {
        res.assume_init()
    }, 0);
}
