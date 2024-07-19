use random_number;
use rodio::*;
use std::fs;
use std::io::{self, BufReader, Read, Write};
use std::path::Path;
use std::process::{exit, Command};
use std::thread;
use std::sync::{self, Arc, Mutex};
fn main() {
    let mut playarg = false;
    let args: Vec<String> = std::env::args().collect();
    if *&args.len() >=2 {
        playarg = true
    }
    if playarg == true {
        if &args[1] != ""{
            let to_play = &args[1];
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let audio = BufReader::new(fs::File::open(to_play).unwrap());
            let source = Decoder::new(audio).expect(
                "Error decoding audio: ",
            );
            let sink = Sink::try_new(&stream_handle).unwrap();
            sink.append(source);
            sink.sleep_until_end();
            return;
        }
    }
    let mut repeat = false;
    loop {
        let songs = fs::read_dir(std::env::var("HOME").unwrap() + "/Music/").expect("You don't even have a music directory!");
        let song_names: Vec<String> = songs
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                if path.is_file() {
                    path.file_name()?.to_str().map(|s| s.to_owned())
                } else {
                    None
                }
            })
            .collect();
        if song_names.len() == 0 {
            println!("You have no songs in your library!");
            exit(1);
        }
        println!("What do you want to do? (shuffle, play, download, library, toggle repeat)");
        let mut order = String::new();
        io::stdin().read_line(&mut order).unwrap();
        order = order.replace("\n", "");
        if order == "toggle repeat" {
            if repeat == false {
                println!("Repeat enabled.");
                repeat = true;
                continue;
            }
            if repeat == true {
                println!("Repeat disabled.");
                repeat = false;
                continue;
            }
        }
        if order == "library" {
            print!("{}[2J", 27 as char);
            println!("Here are the songs currently in your library:");
            for song in &song_names {
                println!("{}", song.replace(".mp3", ""));
            }
        }
        if order == "download" {
            println!("Enter a youtube URL (needs yt-dlp installed):");
            let mut url = String::new();
            if Path::exists(Path::new("/usr/bin/yt-dlp")) == false {
                println!("You need yt-dlp installed!");
                exit(1);
            }
            io::stdin().read_line(&mut url).unwrap();
            Command::new("yt-dlp")
                .current_dir(std::env::var("HOME").unwrap() + "/Music")
                .args(["-x", "--audio-format", "mp3", "--audio-quality", "0", &url])
                .status()
                .expect("Failed to download youtube video/song: ");
        }
        if order == "shuffle" {
            loop {
                let range = 0..song_names.len();
                let rand = random_number::random_ranged(range);
                let random_song = &song_names[rand];
                loop {
                    let to_play = std::env::var("HOME").unwrap() + "/Music/" + random_song;
                    println!("Playing {}", to_play);
                    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                    let audio = BufReader::new(fs::File::open(to_play).unwrap());
                    let source = Decoder::new(audio).expect(
                        "Error decoding audio (probably wrong file type (we only support mp3s)): ",
                    );
                    let sink = Sink::try_new(&stream_handle).unwrap();
                    sink.append(source);
                    sink.sleep_until_end();
                    break;
                }
            }
        }
        if order == "play" {
            let mut file = String::new();
            println!("What song do you want to play?");
            io::stdin().read_line(&mut file).unwrap();
            file = file.replace("\n", "");
            file = file.to_lowercase();
            for song in &song_names {
                let filepath = std::env::var("HOME").unwrap()
                    + "/Music/"
                    + file.to_lowercase().as_str()
                    + ".mp3";
                let path = Path::new(filepath.as_str());
                let name = file.clone().to_lowercase() + ".mp3";
                match Path::exists(path) {
                    true => (),
                    false => {
                        println!(
                            "Can't find {} in {}, would you like me to list the available songs?",
                            name,
                            std::env::var("HOME").unwrap() + "/Music/"
                        );
                        let mut answer = String::new();
                        io::stdin().read_line(&mut answer).unwrap();
                        let answer_chars: Vec<char> = answer.chars().collect();
                        if answer_chars[0] == 'y' {
                            for s in &song_names {
                                println!("{}", s.replace(".mp3", ""));
                            }
                            break;
                        } else {
                            break;
                        }
                    }
                }
                if file.clone().to_lowercase() + ".mp3" == song.to_lowercase() {
                    // play song
                    println!("Playing {}", file);
                    loop {
                        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                        let filepath = std::env::var("HOME").unwrap()
                            + "/Music/"
                            + file.to_lowercase().as_str()
                            + ".mp3";
                        let audio = BufReader::new(fs::File::open(filepath).unwrap());
                        let source = Decoder::new(audio).unwrap();
                        let sink = Sink::try_new(&stream_handle).unwrap();
                        sink.append(source);
                        let sink = Arc::new(Mutex::new(sink));

                        let sink_clone = Arc::clone(&sink);
                        let input_thread = thread::spawn(move|| {
                            println!("P - Pause/Unpause");
                            let mut paused = false;
                            loop {
                                    let mut action = String::new();
                                    io::stdin().read_line(&mut action).unwrap();
                                    action = action.replace("\n", "");
                                    let mut sink = sink_clone.lock().unwrap();
                                    print!("{}", action);
                                    if action.to_lowercase() == "p" {
                                        if paused == true {
                                            sink.play();
                                            paused = false;
                                            continue;
                                        }
                                        if paused == false {
                                            sink.pause();
                                            paused = true;
                                            continue;
                                        }
                                        if action.to_lowercase() == "l" {
                                            sink.set_volume(sink.volume() - 0.05);
                                            println!("Lowered volume by 5.");
                                        }
                                }
                            }
                        });
                        input_thread.join().unwrap();
                        let sink = sink.lock().unwrap();
                        sink.sleep_until_end();
                        if repeat != true {
                            break;
                        }
                    }
                }
            }
        }
    }
}
