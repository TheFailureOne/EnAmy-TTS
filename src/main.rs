// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};

use std::error::Error;
use std::process::Command;
use std::io::Read;
// use std::thread::spawn;
use curl::easy::Easy;
use curl::easy::List;
use serde::{Deserialize, Serialize};
// use slint::platform::{Key, WindowEvent};
use slint::ToSharedString;
use regex::Regex;
use dirs::config_dir;

slint::include_modules!();



fn main() -> Result<(), Box<dyn Error>> {
    let _lock = BufWriter::new(File::create("/tmp/enamy/lock").unwrap());
    let _input_active = BufWriter::new(File::create("/tmp/enamy/inputActive").unwrap());
    let site = "http://192.168.122.193:5000/api/HotelBooking/CreateVoice";
    const STARTING_HORIZONTAL_LIMIT: f32 = 1320.0;
    

    while !fs::metadata("/tmp/enamy/lock").is_err() {
        if !fs::metadata("/tmp/enamy/inputActive").is_err(){
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
                move || {
                    let ui = ui_handle.unwrap();
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
                move || {
                    let ui = ui_handle.unwrap();
                    let c = ui.get_textBox().clone();
                    let a = replace_with_config(c.to_string());
                    
                    let text = format!("{{ \"line\": \"{a}\" }}");
                    let mut data = text.as_bytes();
                    let mut easy = Easy::new();
                    easy.url(site).expect("Failed to set URL");
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

                    Command::new("play")
                        .arg("/tmp/enamy/response.wav")
                        .spawn().unwrap();

                    ui.set_textBox("".to_shared_string());
                    ui.invoke_editedText();

                    //weak.unwrap().window().hide().unwrap();

                }
            });

            ui.on_closeWindow({
                let ui_handle = ui.as_weak();
                move || {
                    let ui = ui_handle.unwrap();
                    if ui.get_keyDown().text == "\u{1b}"{
                        Command::new("rm")
                        .arg("/tmp/enamy/inputActive").spawn().unwrap();

                        ui.window().hide().unwrap();
                    }
                    if ui.get_keyDown().text == "\u{10}" {
                        Command::new("rm")
                        .arg("/tmp/enamy/lock").spawn().unwrap();
                        println!("lock removed");
                    }
                }
            });
            
            ui.run()?;
        }
        else{
            
        }
        
    }
    Ok(())
}

fn replace_with_config(mut text: String) -> String {
    let data = fs::read_to_string(config_dir().unwrap().join("enamy").join("config.json")).expect("Unable to read file");
    #[derive(Serialize, Deserialize, Debug)]
    struct Dict {
        meaning: String,
        word: String,
    }
    #[derive(Serialize, Deserialize, Debug)]
    struct Config {
        dictionary: Vec<Dict>,
    }
    let deserialized: Config = serde_json::from_str(&data).unwrap();
    for i in 0..deserialized.dictionary.len() {
        let regex = format!("\\b({}+)\\b", deserialized.dictionary[i].word);
        let re = Regex::new(&regex).unwrap();
        text = re.replace_all(text.as_str(), &deserialized.dictionary[i].meaning).to_string();
    }
    return text;
}