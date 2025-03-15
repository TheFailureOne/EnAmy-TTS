// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs::{self, read};
use std::fs::File;
use std::io::{BufWriter, Write, BufReader};
use std::thread;
use std::time::Duration;
use rodio::{Decoder, OutputStream, Sink, Source};


use std::error::Error;
use std::io::Read;
// use std::thread::spawn;
use curl::easy::Easy;
use curl::easy::List;
use serde::{Deserialize, Serialize};
// use slint::platform::{Key, WindowEvent};
use slint::ToSharedString;
use regex::Regex;
use dirs::config_dir;
use global_hotkey::{GlobalHotKeyManager, hotkey::{HotKey, Modifiers, Code}, GlobalHotKeyEvent};
use std::sync::atomic::{AtomicBool, Ordering}; 
use std::sync::{Arc, Mutex};


slint::include_modules!();

#[derive(Serialize, Deserialize, Debug)]
struct Dict {
    meaning: String,
    word: String,
}
#[derive(Serialize, Deserialize, Debug)]
struct Config {
    server_url: String,
    typing_style: String,
    dictionary: Vec<Dict>,
}


fn main() -> Result<(), Box<dyn Error>> {
    let lock = Arc::new(Mutex::new(AtomicBool::new(true)));
    let input_active = Arc::new(Mutex::new(AtomicBool::new(true)));
    let started_input = Arc::new(Mutex::new(AtomicBool::new(true)));
    let new_file = Arc::new(Mutex::new(AtomicBool::new(false)));
    started_input.lock().unwrap().store(false, Ordering::SeqCst);
    const STARTING_HORIZONTAL_LIMIT: f32 = 1320.0;
    let manager = GlobalHotKeyManager::new().unwrap();
    let hotkey = HotKey::new(Some(Modifiers::CONTROL), Code::Enter);
    let mut text_input = Arc::new(Mutex::new(String::new()));
    *text_input.lock().unwrap() = "".to_string();

    let _audio_thread = thread::spawn({
        let new_file_lock = new_file.clone();
        move ||{
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();
            let mut is_playing = false;
            let mut play_head_pos = Duration::new(0, 0);

            loop{
                // println!("{:?}", play_head_pos);
                if new_file_lock.lock().unwrap().load(Ordering::Relaxed) == true {
                    println!("Setting it");
                    // println!("{:?}", play_head_pos);
                    println!("Set");
                    if sink.empty() || read_config().typing_style != "FluentSpeech" {
                        play_head_pos = Duration::new(0, 0);
                    }
                    let file = BufReader::new(File::open("/tmp/enamy/response.wav").unwrap());
                    let source = Decoder::new(file).unwrap();
                    println!("playing audio");
                    play_head_pos = sink.get_pos();
                    sink.append(source.skip_duration(sink.get_pos()));
                    // if(is_playing == true){
                    //     sink.skip_one();
                    // }
                    // is_playing = true;
                    new_file_lock.lock().unwrap().store(false, Ordering::SeqCst);
                    sink.play();
                }
            }
        }});
    let _count_down_thread = thread::spawn({ 
        let input_lock = started_input.clone();
        let text_lock = text_input.clone();
        let new_file_lock = new_file.clone();
        move || {
            let mut count_down = 5.0;
            if read_config().typing_style != "FluentSpeech" {
                return 0;
            }
            loop{
                if input_lock.lock().unwrap().load(Ordering::Relaxed) == true {
                    thread::sleep(std::time::Duration::from_secs_f64(count_down));
                    println!("Sending requests and all");
                    let a = replace_with_config(text_lock.lock().unwrap().clone());
    
                    let text = format!("{{ \"line\": \"{a}\" }}");
                    let new_file_val = play_tts(text);
                    new_file_lock.lock().unwrap().store(new_file_val, Ordering::SeqCst);
                    count_down = 1.5;
                }
            }
        }
    });


    while lock.lock().unwrap().load(Ordering::Relaxed) == true {
        if input_active.lock().unwrap().load(Ordering::Relaxed) == true {
            match fs::metadata("/tmp/enamy") {
                Ok(_) => print!("All good, tmp folder exists"),
                Err(_) => fs::create_dir("/tmp/enamy").expect("Failed to create folder"),
            }
            let ui = AppWindow::new()?;
            let mut horizontal_limit = STARTING_HORIZONTAL_LIMIT;
            let mut horizontal_width = ui.get_rectWidth();
            ui.set_rectWidth(30.0);
            ui.set_rectX(235.0);
            ui.set_rectHeight(90.0);
            ui.on_editedText({
                let ui_handle = ui.as_weak();
                let input_lock = started_input.clone();
                let text_lock = text_input.clone();
                move || {
                    let ui = ui_handle.unwrap();

                    if ui.get_textBox() == "" {
                        input_lock.lock().unwrap().store(false, Ordering::SeqCst);
                    }
                    else{
                        input_lock.lock().unwrap().store(true, Ordering::SeqCst);
                    }
                    *text_lock.lock().unwrap() = ui.get_textBox().to_string();
                    
                    horizontal_width = 30.0 + (30.0 * ui.get_textBox().len() as f32);
                    if horizontal_width <= STARTING_HORIZONTAL_LIMIT {
                        ui.set_rectWidth(horizontal_width);
                    }
                    else if horizontal_width >= horizontal_limit {
                        ui.set_rectHeight(ui.get_rectHeight() + 90.0);
                        horizontal_limit += STARTING_HORIZONTAL_LIMIT;
                    }
                    if horizontal_width <= horizontal_limit - STARTING_HORIZONTAL_LIMIT{
                        ui.set_rectHeight(ui.get_rectHeight() - 90.0);
                        horizontal_limit -= STARTING_HORIZONTAL_LIMIT;
                    }
                    ui.set_rectX(235.0 - (15.0 * ui.get_textBox().len() as f32));
                }
            });
            //let weak = ui.as_weak();

            ui.on_sendRequest({ 
                let ui_handle = ui.as_weak();
                let lock_lock = lock.clone();
                let new_file_lock = new_file.clone();
                move || {
                    let ui = ui_handle.unwrap();
                    
                    if ui.get_textBox() == "cmd::quit" {
                        lock_lock.lock().unwrap().store(false, Ordering::SeqCst);

                        ui.window().hide().unwrap();
                    }
                    else {          
                        let c = ui.get_textBox().clone();
                        let a = replace_with_config(c.to_string());
    
                        let text = format!("{{ \"line\": \"{a}\" }}");

                        let new_file_val = play_tts(text);

                        new_file_lock.lock().unwrap().store(new_file_val, Ordering::SeqCst);      

                        ui.set_textBox("".to_shared_string());
                        ui.invoke_editedText();
                    }

                    //weak.unwrap().window().hide().unwrap();

                }
            });

            ui.on_closeWindow({
                let ui_handle = ui.as_weak();
                let input_active_lock = input_active.clone();
                // let test_handle = &test;
                move || {
                    let ui = ui_handle.unwrap();
                    if ui.get_keyDown().text == "\u{1b}"{
                        input_active_lock.lock().unwrap().store(false, Ordering::SeqCst);

                        ui.window().hide().unwrap();
                    }
                }
            });
            
            ui.run()?;
        }
        else{
            let _ = manager.register(hotkey);

            if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv(){
                println!("{:?}", event);
                input_active.lock().unwrap().store(true, Ordering::SeqCst);
            }
        }
    }
    Ok(())
}

fn replace_with_config(mut text: String) -> String {
    let deserialized = read_config();

    for i in 0..deserialized.dictionary.len() {
        let regex = format!("\\b({}+)\\b", deserialized.dictionary[i].word);
        let re = Regex::new(&regex).unwrap();
        text = re.replace_all(text.as_str(), &deserialized.dictionary[i].meaning).to_string();
    }
    return text;
}

fn read_config() -> Config{
    let data = fs::read_to_string(config_dir().unwrap().join("enamy").join("config.json")).expect("Unable to read file");
    let deserialized: Config = serde_json::from_str(&data).unwrap();

    return deserialized;
}

fn play_tts(text: String) -> bool{
    let url = read_config().server_url;
    let site = format!("{url}/api/HotelBooking/CreateVoice");
    let mut data = text.as_bytes();
    let mut easy = Easy::new();
    easy.url(site.as_str()).expect("Failed to set URL");
    easy.post(true).unwrap();
    easy.post_field_size(data.len() as u64).unwrap();
    let mut header = List::new();
    header.append("Content-Type: application/json").expect("Failed to add header");
    easy.http_headers(header).unwrap();
    let mut file = BufWriter::new(File::create("/tmp/enamy/response.wav").unwrap());

    easy.write_function(move |data| {
    file.write_all(data).unwrap();
    Ok(data.len())
    }).unwrap();


    let mut transfer = easy.transfer();
    transfer.read_function(|buf| {
        Ok(data.read(buf).unwrap_or(0))
    }).unwrap();
    transfer.perform().unwrap();
    return true;
}