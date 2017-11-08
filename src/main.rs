use std::env;
use std::process::Command;

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();

    // run the linker exactly as `rustc` instructed
    let mut ld1 = Command::new("arm-none-eabi-ld");
    ld1.args(&args);
    eprintln!("{:?}", ld1);
    assert!(ld1.status().unwrap().success());

    // retrieve the output file name
    let mut output = None;
    let mut iargs = args.iter();
    while let Some(arg) = iargs.next() {
        if arg == "-o" {
            output = iargs.next();
            break;
        }
    }

    let output = output.unwrap();

    // shell out to `size` to get the size of the linker sections
    // TODO use a library instead of calling `size` (?)
    let mut size = Command::new("arm-none-eabi-size");
    size.arg("-A").arg(output);
    eprintln!("{:?}", size);
    let stdout = String::from_utf8(size.output().unwrap().stdout).unwrap();

    // parse the stdout of `size`
    let mut bss = None;
    let mut data = None;
    let mut sram = None;
    let mut ram = None;
    for line in stdout.lines() {
        if line.starts_with(".bss") {
            // .bss $bss 0x20000000
            bss = line.split_whitespace()
                .nth(1)
                .map(|s| s.parse::<u32>().unwrap());
        } else if line.starts_with(".data") {
            // .data $data 0x20000010
            data = line.split_whitespace()
                .nth(1)
                .map(|s| s.parse::<u32>().unwrap());
        } else if line.starts_with(".stack") {
            // .stack $ram $sram
            let mut parts = line.split_whitespace().skip(1);
            ram = parts.next().map(|s| s.parse::<u32>().unwrap());
            sram = parts.next().map(|s| s.parse::<u32>().unwrap());
        }
    }

    // compute the new start address of the (.bss+.data) section
    // the relocated stack will start at that address as well (and grow downwards)
    let bss = bss.unwrap();
    let data = data.unwrap();
    let sram = sram.unwrap();
    let ram = ram.unwrap();
    let eram = sram + ram;

    let sbss = eram - bss - data;

    let mut ld2 = Command::new("arm-none-eabi-ld");
    ld2.arg(format!("--defsym=_sbss={}", sbss))
        .arg(format!("--defsym=_stack_start={}", sbss))
        .args(&args);
    eprintln!("{:?}", ld2);
    assert!(ld2.status().unwrap().success());
}
