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
    
    file_mmap.flush().unwrap();
}

fn replace(old: &[u8], new: &[u8], buffer: &mut [u8]) {
    let mut buffer_index = 0;
    let buffer_len = buffer.len();
    let old_len = old.len();
    let new_len = new.len();
    let gap_len = old_len - new_len;
    
    while buffer_index + old_len < buffer_len {
        if buffer[buffer_index..buffer_index+old_len] == *old {
            // TODO: handle no terminator
            for new_index in 0..new_len {
                buffer[buffer_index] = new[new_index];
                buffer_index += 1;
            }
            while buffer[buffer_index + gap_len] != 0 {
                buffer[buffer_index] = buffer[buffer_index + gap_len];
                buffer_index += 1;
            }
            while buffer[buffer_index] != 0 {
                buffer[buffer_index] = 0;
                buffer_index += 1;
            }
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
        let mut buffer = create_buffer(b"abcd\0e");
        replace(b"bc", b"yz", &mut buffer);
        let expected = create_buffer(b"ayzd\0e");
        assert_eq!(expected, buffer);
    }
    
    #[test]
    fn when_old_is_longer_than_new_and_there_are_no_trailing_characters_then_string_is_padded_with_null() {
        let mut buffer = create_buffer(b"abcde\0f");
        replace(b"bcde", b"yz", &mut buffer);
        let expected = create_buffer(b"ayz\0\0\0f");
        assert_eq!(expected, buffer);
    }
    
    #[test]
    fn when_old_is_longer_than_new_and_there_are_characters_after_old_then_trailing_characters_are_shifted() {
        let mut buffer = create_buffer(b"abcdefg\0h");
        replace(b"bcde", b"yz", &mut buffer);
        let expected = create_buffer(b"ayzfg\0\0\0h");
        assert_eq!(expected, buffer);
    }
    
    fn create_buffer(value: &[u8]) -> Vec<u8> {
        return Vec::from(value);
    }
}
