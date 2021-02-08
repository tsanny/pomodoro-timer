use std::{
    error::Error,
    include_bytes,
    io::{self, Write},
    process::{Command, Stdio},
    str::FromStr,
    sync::mpsc::channel,
    thread::{sleep, spawn},
    time::Duration,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut input_interval_length = String::from_str("5")?;
    let mut input_interval_count = String::from_str("2")?;

    // clear terminal and set cursor to top
    //print!("\x1B[2J\x1B[1;1H");
    //print!("Enter the length in seconds of each interval (max 65535): ");
    //// ensure prompt above is printed immediately
    //io::stdout().flush()?;
    //io::stdin().read_line(&mut input_interval_length)?;

    //print!("Enter the count of intervals (max 255): ");
    //io::stdout().flush()?;
    //io::stdin().read_line(&mut input_interval_count)?;

    print!("\x1B[2J\x1B[1;1H");
    // convert input to numbers
    let interval_length: u16 = input_interval_length
        .trim()
        .parse()
        .expect("Min, max interval length (seconds) of 1,65535");
    let interval_count: u8 = input_interval_count
        .trim()
        .parse()
        .expect("Min, max interval count of 1,255");

    // create transmitters and receivers to communicate between threads
    let (time_tx, time_rx) = channel();
    let (audio_tx, audio_rx) = channel();

    // spawn audio thread
    spawn(move || {
        // store sound as part of binary
        let test = include_bytes!("./ding.wav");
        let mut audio = Command::new("sh");
        audio.args(&["-c", "aplay"]);
        // set up pipe to send sound through
        audio.stdin(Stdio::piped());
        // ignore output of aplay
        audio.stderr(Stdio::null());
        loop {
            // when signal received, play sound
            let _ = audio_rx.recv().map(|_reply| {
                audio
                    .spawn()
                    .expect("failed to spawn aplay, is it installed?")
                    .stdin
                    .as_ref()
                    .unwrap()
                    .write_all(test)
                    .unwrap();
            });
        }
    });

    spawn(move || {
        // initial warm-up time
        let mut time_sec = 10;
        loop {
            sleep(Duration::from_secs(1));
            // send new time early to be displayed
            time_tx.send(time_sec - 1).unwrap();
            time_sec -= 1;
            if time_sec == 0 {
                // reset timer to interval length again
                time_sec = interval_length;
            }
        }
    });

    let mut current_interval = 0;
    loop {
        let _ = time_rx.recv().map(|reply| {
            // get minutes as int, so if under 1 minute shows 0
            let interval_minutes = reply / 60;
            print!("\x1B[2J\x1B[1;1H");
            // format time to have 0 before int when below 2sf
            println!(
                "Interval {} of {}: {:02}:{:02}",
                current_interval,
                interval_count,
                interval_minutes,
                reply - (interval_minutes * 60)
            );

            // countdown for this interval has reached zero, play sound and move on
            if reply == 0 {
                audio_tx.send(true).unwrap();
                current_interval += 1;
                // when current == total + 1 (warm-up), we are done, so exit
                if current_interval == interval_count + 1 {
                    println!("Finished!");
                    // wait until sound finishes (2.856 seconds)
                    sleep(Duration::from_millis(2856));
                    std::process::exit(0);
                }
            }
        });
    }
}
