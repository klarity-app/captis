use captis::*;
use std::env;

fn main() {
    let mut args = env::args();
    args.next().unwrap();
    let capturer = init_capturer().unwrap();
    println!("Found Displays: {:?}", capturer.displays());
    while let Some(num) = args.next() {
        let num: usize = num.parse().unwrap();
        let now = std::time::Instant::now();
        for _ in 0..60 {
            capturer.capture(num).unwrap();
        }
        let image = capturer.capture(num).unwrap();
        println!("Captures 60 frames in {}ms", now.elapsed().as_millis(),);
        let name = format!("test-{}.jpg", num);
        image.save(&name).unwrap();
        println!("Saved {}", name);
    }
}
