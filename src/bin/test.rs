use captis::*;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args();

    args.next().unwrap();

    let capturer = init_capturer()?;

    println!("Found Displays: {:?}", capturer.displays());

    while let Some(num) = args.next() {
        let num: usize = num.parse()?;

        let now = std::time::Instant::now();

        for _ in 0..60 {
            capturer.capture(num)?;
        }

        let image = capturer.capture(num)?;

        println!("Captures 60 frames in {}ms", now.elapsed().as_millis(),);

        let name = format!("test-{}.jpg", num);

        image.save(&name)?;

        println!("Saved {}", name);
    }

    Ok(())
}
