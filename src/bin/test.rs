use captis::*;
use std::env;

fn main() {
    if cfg!(target_os = "windows") || cfg!(target_os = "linux") {
        let mut args = env::args();
        args.next().unwrap();
        let capturer = init_capturer().unwrap();
        println!("Found Displays: {:?}", capturer.displays());
        while let Some(num) = args.next() {
            let num: usize = num.parse().unwrap();
            let now = std::time::Instant::now();
            let image = capturer.capture(num).unwrap();
            image.save(format!("test-{}.jpeg", num)).unwrap();
            println!("Elapsed: {}, Captured: {}", now.elapsed().as_millis(), num);
        }
    }
}
