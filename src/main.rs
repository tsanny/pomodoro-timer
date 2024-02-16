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
    /// The length of each focus interval in seconds, minimum 1
    #[structopt(default_value = "10")]
    focus_interval_length: u16,

    /// The length of each rest interval in seconds, minimum 1
    #[structopt(default_value = "5")]
    rest_interval_length: u16,

    /// The number of intervals (after warm up), minimum 1
    #[structopt(default_value = "3")]
    interval_count: u8,

    /// The warmup time in seconds, minimum 1
    #[structopt(default_value = "5")]
    warmup_time: u16,
}

fn main() {
    let args = Cli::from_args();
    let focus_interval_length: u16 = 1.max(args.focus_interval_length);
    let rest_interval_length: u16 = 1.max(args.rest_interval_length);
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
    let mut is_rest_time = false;
    // initial warm-up time
    let mut time_sec = warmup_time;
    let mut session_type = "Focus";
    let tick = periodic_ms(1000);
    loop {
        tick.recv().unwrap();
        time_sec -= 1;

        // get minutes as int, so if under 1 minute shows 0
        let interval_minutes = time_sec / 60;
        // clear terminal and set cursor to start of first line
        print!("\x1B[2J\x1B[1;1H");
        // format time to have 0 before int when below 2sf
        let interval_text = format!(
            "{} {} of {}",
            session_type,
            current_interval,
            interval_count,
        ); 
        let timer_text = format!(
            "{:02}:{:02}",
            interval_minutes,
            time_sec - (interval_minutes * 60)
        );
        // pass formatted time to figlet and print
        let interval_figure = standard_font.convert(interval_text.as_str());
        let timer_figure = standard_font.convert(timer_text.as_str());
        println!("{}", interval_figure.unwrap());
        println!("{}", timer_figure.unwrap());

        // countdown for this interval has reached zero, play sound and move on
        if time_sec == 0 {
            // reset timer to interval length again
            if is_rest_time {
                time_sec = rest_interval_length;
                session_type = "Rest";
            } else {
                time_sec = focus_interval_length;
                session_type = "Focus";
                current_interval += 1;
            }
            is_rest_time ^= true;

            // audio_tx.send(()).unwrap();
            audio_tx.send(());
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
