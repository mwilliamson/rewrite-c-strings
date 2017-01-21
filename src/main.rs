extern crate clap;
extern crate memmap;
use clap::{Arg, App};
use memmap::{Mmap, Protection};
use std::os::unix::ffi::OsStrExt;

fn main() {
    let matches = App::new("rewrite-c-strings")
        .arg(Arg::with_name("old").required(true))
        .arg(Arg::with_name("new").required(true))
        .arg(Arg::with_name("file").required(true))
        .get_matches();
    
    let old = matches.value_of_os("old").unwrap().as_bytes();
    let new = matches.value_of_os("new").unwrap().as_bytes();
    let path = matches.value_of_os("file").unwrap();
    
    let mut file_mmap = Mmap::open_path(path, Protection::ReadWrite).unwrap();
    
    unsafe {
        replace(old, new, file_mmap.as_mut_slice());
    }
    
    file_mmap.flush();
}

fn replace(old: &[u8], new: &[u8], buffer: &mut [u8]) {
    let mut buffer_index = 0;
    let buffer_len = buffer.len();
    let old_len = old.len();
    
    while buffer_index + old_len < buffer_len {
        let mut buffer_slice = &mut buffer[buffer_index..buffer_index+old_len];
        if buffer_slice == old {
            buffer_slice.copy_from_slice(new);
            buffer_index += old_len;
        } else {
            buffer_index += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::replace;
    
    #[test]
    fn empty_buffer_is_unchanged() {
        let buffer: &mut [u8] = &mut [];
        replace(b"old", b"new", buffer.as_mut());
        let expected: &[u8] = &[];
        assert_eq!(expected, buffer);
    }
    
    #[test]
    fn when_buffer_does_not_contain_old_then_buffer_is_unchanged() {
        let mut buffer = create_buffer(b"string");
        replace(b"old", b"new", &mut buffer);
        let expected = create_buffer(b"string");
        assert_eq!(expected, buffer);
    }
    
    #[test]
    fn when_old_is_same_length_as_new_then_replacement_occurs() {
        let mut buffer = create_buffer(b"abcde");
        replace(b"bc", b"yz", &mut buffer);
        let expected = create_buffer(b"ayzde");
        assert_eq!(expected, buffer);
    }
    
    fn create_buffer(value: &[u8]) -> Vec<u8> {
        return Vec::from(value);
    }
}
