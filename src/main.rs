use figlet_rs::FIGfont;
use schedule_recv::{oneshot_ms, periodic_ms};
use std::{
    include_bytes,
    io::Write,
    process::{exit, Command, Stdio},
    sync::mpsc::channel,
    thread::spawn,
};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    /// The length of each interval in seconds, minimum 1
    #[structopt(default_value = "60")]
    interval_length: u16,
    /// The number of intervals (after warm up), minimum 1
    #[structopt(default_value = "2")]
    interval_count: u8,

    /// The warmup time in seconds, minimum 1
    #[structopt(short, long, default_value = "10")]
    warmup_time: u16,
}

fn main() {
    let args = Cli::from_args();
    let interval_length: u16 = 1.max(args.interval_length);
    let interval_count: u8 = 1.max(args.interval_count);
    let warmup_time: u16 = 1.max(args.warmup_time);

    // set up figlet with standard font
    let standard_font = FIGfont::standand().unwrap();

    // create transmitter and receiver to communicate between threads
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
                    .unwrap()
                    .write_all(test)
                    .unwrap();
            });
        }
    });

    let mut current_interval = 0;
    // initial warm-up time
    let mut time_sec = warmup_time;
    let tick = periodic_ms(1000);
    loop {
        tick.recv().unwrap();
        time_sec -= 1;

        // get minutes as int, so if under 1 minute shows 0
        let interval_minutes = time_sec / 60;
        // clear terminal and set cursor to start of first line
        print!("\x1B[2J\x1B[1;1H");
        // format time to have 0 before int when below 2sf
        let text = format!(
            "Interval {} of {}: {:02}:{:02}",
            current_interval,
            interval_count,
            interval_minutes,
            time_sec - (interval_minutes * 60)
        );
        // pass formatted time to figlet and print
        let figure = standard_font.convert(text.as_str());
        println!("{}", figure.unwrap());

        // countdown for this interval has reached zero, play sound and move on
        if time_sec == 0 {
            // reset timer to interval length again
            time_sec = interval_length;

            audio_tx.send(()).unwrap();
            current_interval += 1;
            // when current == total + 1 (warm-up), we are done, so exit
            if current_interval == interval_count + 1 {
                println!("Finished!");
                // wait until sound finishes (0.9 seconds)
                oneshot_ms(900).recv().unwrap();
                exit(0);
            }
        }
    }
}
