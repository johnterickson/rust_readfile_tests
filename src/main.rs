use std::io::Read;
use std::fs::File;

// with a smaller buffer, there's basically no difference between the methods...
// const BUFFER_SIZE: usize = 2 * 1024;

// ...but the larger the Vec, the bigger the discrepancy.
// for simplicity's sake, let's assume this is a hard upper limit.
const DEFAULT_BUFFER_SIZE: usize = 300 * 1024 * 1024;

const ITERATIONS: usize = 1000;

fn naive(buffer_size: usize) {
    let mut buffer = Vec::with_capacity(buffer_size);

    for _ in 0..ITERATIONS {
        let mut file = File::open("some_1kb_file.txt").expect("opening file");

        let metadata = file.metadata().expect("reading metadata");
        let len = metadata.len();
        assert!(len <= buffer_size as u64);

        buffer.clear();
        file.read_to_end(&mut buffer).expect("reading file");

        // do "stuff" with buffer
        let check = buffer.iter().take(len.try_into().unwrap()).fold(0usize, |acc, x| acc.wrapping_add(*x as usize));

        println!("length: {len}, check: {check}");
    }
}

fn take(buffer_size: usize) {
    let mut buffer = Vec::with_capacity(buffer_size);

    for _ in 0..ITERATIONS {
        let file = File::open("some_1kb_file.txt").expect("opening file");

        let metadata = file.metadata().expect("reading metadata");
        let len = metadata.len();
        assert!(len <= buffer_size as u64);

        buffer.clear();
        file.take(len).read_to_end(&mut buffer).expect("reading file");

        // this also behaves like the straight `read_to_end` with a significant slowdown:
        // file.take(BUFFER_SIZE as u64).read_to_end(&mut buffer).expect("reading file");

        // do "stuff" with buffer
        let check = buffer.iter().take(len.try_into().unwrap()).fold(0usize, |acc, x| acc.wrapping_add(*x as usize));

        println!("length: {len}, check: {check}");
    }
}

fn exact(buffer_size: usize) {
    let mut buffer = vec![0u8; buffer_size];

    for _ in 0..ITERATIONS {
        let mut file = File::open("some_1kb_file.txt").expect("opening file");

        let metadata = file.metadata().expect("reading metadata");
        let len = metadata.len() as usize;
        assert!(len <= buffer_size);

        // SAFETY: initialized by `vec!` and within capacity by `assert!`
        unsafe { buffer.set_len(len); }
        file.read_exact(&mut buffer[0..len]).expect("reading file");

        // do "stuff" with buffer
        let check = buffer.iter().take(len).fold(0usize, |acc, x| acc.wrapping_add(*x as usize));

        println!("length: {len}, check: {check}");
    }
}

fn exact_slice(buffer_size: usize) {
    let mut buffer = vec![0u8; buffer_size];
    let buffer = &mut buffer[..];

    for _ in 0..ITERATIONS {
        let mut file = File::open("some_1kb_file.txt").expect("opening file");

        let metadata = file.metadata().expect("reading metadata");
        let len = metadata.len() as usize;
        assert!(len <= buffer_size);

        // SAFETY: initialized by `vec!` and within capacity by `assert!`
        file.read_exact(&mut buffer[0..len]).expect("reading file");

        // do "stuff" with buffer
        let check = buffer.iter().take(len).fold(0usize, |acc, x| acc.wrapping_add(*x as usize));

        println!("length: {len}, check: {check}");
    }
}

fn exact_slice_hide_backing_slice(buffer_size: usize) {
    let mut buffer = vec![0u8; buffer_size];
    let buffer = &mut buffer[..];

    for _ in 0..ITERATIONS {
        let mut file = File::open("some_1kb_file.txt").expect("opening file");

        let metadata = file.metadata().expect("reading metadata");
        let len = metadata.len() as usize;
        assert!(len <= buffer_size);

        unsafe
        {
            let buffer_ptr = &mut buffer[0] as *mut u8;
            let unsafe_slice = std::slice::from_raw_parts_mut(buffer_ptr, len);
            file.read_exact(unsafe_slice).expect("reading file");
        }

        // do "stuff" with buffer
        let check = buffer.iter().take(len).fold(0usize, |acc, x| acc.wrapping_add(*x as usize));

        println!("length: {len}, check: {check}");
    }
}

fn whole_slice(buffer_size: usize) {
    let mut buffer = vec![0u8; buffer_size];
    let buffer = &mut buffer[..];

    for _ in 0..ITERATIONS {
        let mut file = File::open("some_1kb_file.txt").expect("opening file");

        let metadata = file.metadata().expect("reading metadata");
        let len = metadata.len() as usize;
        assert!(len <= buffer_size);

        // SAFETY: initialized by `vec!` and within capacity by `assert!`
        assert_eq!(len, file.read(buffer).expect("reading file"));

        // do "stuff" with buffer
        let check = buffer.iter().take(len).fold(0usize, |acc, x| acc.wrapping_add(*x as usize));

        println!("length: {len}, check: {check}");
    }
}

// fn NtReadFileDirect(buffer_size: usize) {
//     use windows_sys::core::*;
//     use windows_sys::Win32::Storage::FileSystem;
//     use windows_sys::Win32::System::IO;

//     let mut buffer = vec![0u8; buffer_size];
//     let buffer = &mut buffer[..];

//     for _ in 0..ITERATIONS {
//         let mut file = File::open("some_1kb_file.txt").expect("opening file");

//         let metadata = file.metadata().expect("reading metadata");
//         let len = metadata.len() as usize;
//         assert!(len <= buffer_size);

//         // SAFETY: initialized by `vec!` and within capacity by `assert!`
//         let mut lpnumberofbytesread;
//         unsafe 
//         {
//             ReadFile(file.into(), buffer as *mut c_void, len.try_into().unwrap(), &mut lpnumberofbytesread, None);
//         }
//         assert_eq!(len, file.read(buffer).expect("reading file"));

//         // do "stuff" with buffer
//         let check = buffer.iter().fold(0usize, |acc, x| acc.wrapping_add(*x as usize));

//         println!("length: {len}, check: {check}");
//     }
// }

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("usage: {} <method>", args[0]);
        return;
    }

    let buffer_size = if let Some(buffer_size) = args.get(2) {
        buffer_size.parse::<usize>().expect("parsing buffer size")
    } else {
        DEFAULT_BUFFER_SIZE
    };

    println!("Using buffer size: {}", buffer_size);

    match args[1].as_str() {
        "naive" => naive(buffer_size),
        "take" => take(buffer_size),
        "exact" => exact(buffer_size),
        "whole_slice" => whole_slice(buffer_size),
        "exact_slice" => exact_slice(buffer_size),
        "exact_slice_hide_backing_slice" => exact_slice_hide_backing_slice(buffer_size),
        _ => println!("Unknown method: {}", args[1]),
    }
}