# Better Interval Timer

This interval timer written in Rust is intended to be a minor improvement over the C interval timer implemented in [this blog post](https://blog.snowvall.xyz/posts/06-02-2021-C-Timings.html).

A true interval timer would also feature rest times between intervals, a configurable warm-up period at the start (hardcoded to 10s at this moment in time), etc. However, those features are out of scope for this repository - at this point in time.

## Implementation

Three explicit threads in total are used:

- The main thread receives user input at the start and then displays the countdown timer for each interval. When it receives a time of 0, it transmits a message to the sound thread to play the sound.
- The time thread tracks the seconds passing by sleeping for one second, transmitting the new time to the main thread, and then doing as little work as possible until the next loop to minimise any drift in the time tracking.
- The sound thread plays a ding sound effect via `aplay` when it receives the message to do so. It does this by spawning a child process of aplay.

Additionally, reasonable limits have been set on the interval length and count (u16 and u8 respectively).

## Performance

Performance was measured by commenting out the code requesting an interval length and count, modifying the input strings in the code instead.

We expect the Ideal Time to be the warmup time (10 seconds) + (interval length * interval count) + sound effect at the end (2.856 seconds).

| Interval length (Seconds) | Interval count | Time (MM:SS.sss) | Ideal Time (MM:SS.ss) | Drift (MM:SS.sss) |       Error (%)       |
| :-----------------------: | :------------: | :--------------: | :-------------------: | :---------------: | :-------------------: |
|            05             |       02       |    00:22.860     |       00:22.856       |     00:00.004     |   0.0175% (3 s.f.)    |
|           900             |       02       |    30:13.086     |       30:12.856       |     00:00.230     | **0.0127% (3 s.f.)**  |

Interestingly, it appears the drift error decreases the longer the program is running.

## Comparison

The blog post states:

> I'll run a proper test at some point, but now over 30 minutes the overall time should be ~30:15.00, which I think is as close as I'm going to get it and especially if you are using it for exercise, the 15 extra seconds won't hurt.

Giving the benefit of the doubt, I'll assume the author has forgotten to take into account the 10 second warm-up and 2.856 second sound at the end in the claim of "15 extra seconds", so the drift was 2.144 seconds. Assuming a length of 1800 seconds and a count of 1 for the blog post:

|        Program        |    Time    | Ideal Time  |   Drift   |        Error         |
| :-------------------: | :--------: | :---------: | :-------: | :------------------: |
|    interval-timer     | ~30:15.000 |  30:12.856  | 00:02.144 |   0.118% (3 s.f.)    |
| better-interval-timer |  30:13.086 |  30:12.856  | 00:00.230 | **0.0127% (3 s.f.)** |

This repository reduces drift error by 89.3% (3 s.f.).
