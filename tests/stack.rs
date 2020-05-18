use baffa::{WriteBuf, WriteBufExt, StaticBuffer, ReadBuf, ReadBufExt, ContBuf};
use core::{mem, slice};

#[test]
fn test_stack_buffer() {
    let num = u64::max_value();

    let bytes = unsafe {
        slice::from_raw_parts(&num as *const _ as *const u8, mem::size_of::<u64>())
    };

    let mut buffer = StaticBuffer::<u64>::new();

    assert_eq!(buffer.as_read_slice().len(), 0);
    assert_eq!(buffer.as_write_slice().len(), 8);

    assert_eq!(buffer.write_slice(bytes), 8);
    assert_eq!(buffer.as_write_slice().len(), 0);
    assert_eq!(buffer.as_slice(), bytes);
    assert_eq!(buffer.as_read_slice(), bytes);
    assert_eq!(buffer.write_slice(bytes), 0);
    assert_eq!(buffer.as_slice(), bytes);
    assert_eq!(buffer.as_read_slice(), bytes);

    let mut res = mem::MaybeUninit::<u64>::new(0);
    assert_eq!(buffer.read_value(&mut res), 8);
    assert_eq!(unsafe { res.assume_init() }, num);

    unsafe {
        buffer.set_len(4);
    }

    assert_eq!(buffer.write_value(&252u32), 4);

    let mut res = mem::MaybeUninit::<u32>::new(0);
    assert_eq!(buffer.read_value(&mut res), 4);
    assert_eq!(unsafe { res.assume_init() }, u32::max_value());

    let mut res = mem::MaybeUninit::<u32>::new(0);
    assert_eq!(buffer.read_value(&mut res), 4);
    assert_eq!(unsafe { res.assume_init() }, 252);

    let mut res = mem::MaybeUninit::<u32>::new(0);
    assert_eq!(buffer.read_value(&mut res), 0);
    assert_eq!(unsafe { res.assume_init() }, 0);

    assert_eq!(buffer.as_write_slice().len(), 8);
    assert_eq!(buffer.write_value(&u32::max_value()), 4);
    assert_eq!(buffer.len(), 4);
    assert_eq!(buffer.as_write_slice().len(), 4);
    for idx in 0..4 {
        assert_eq!(buffer[idx], 255);
    }
    assert_eq!(buffer.write_value(&u32::max_value()), 4);
    assert_eq!(buffer.len(), 8);
    assert_eq!(buffer.as_write_slice().len(), 0);
    for idx in 0..8 {
        assert_eq!(buffer[idx], 255);
    }
    assert_eq!(buffer.as_slice(), bytes);
    assert_eq!(buffer.write_value(&u32::max_value()), 0);
    assert_eq!(buffer.as_slice(), bytes);
    assert_eq!(buffer.as_slice().len(), 8);

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
    for idx in 0..8 {
        assert_eq!(buffer[idx], 255);
    }
    assert_eq!(buffer.write_slice(bytes), 8);
    assert_eq!(buffer.available(), 8);
    assert_eq!(buffer.write_value(&0u32), 4);
    assert_eq!(buffer.available(), 8);
    for idx in 0..4 {
        assert_eq!(buffer[idx], 255);
    }
    for idx in 4..8 {
        assert_eq!(buffer[idx], 0);
    }

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
