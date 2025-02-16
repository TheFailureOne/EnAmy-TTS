// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::fs::File;
use std::fs::DirBuilder;
use std::io::{BufWriter, Write};

use std::error::Error;
use std::process::Command;
use std::io::Read;
use curl::easy::Easy;
use curl::easy::List;
use slint::ToSharedString;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    match fs::metadata("/tmp/enamy") {
        Ok(_) => print!("All good, tmp folder exists"),
        Err(_) => fs::create_dir("/tmp/enamy").expect("Failed to create folder"),
    }
    let ui = AppWindow::new()?;
    const STARTING_HORIZONTAL_LIMIT: f32 = 1320.0;
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
    let weak = ui.as_weak();
    ui.on_closeWindow({ 
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            let a = ui.get_textBox().to_string();
            let text = format!("{{ \"line\": \"{a}\" }}");
            let site = "http://192.168.122.193:5000/api/HotelBooking/CreateVoice";
            let mut data = text.as_bytes();
            let mut easy = Easy::new();
            easy.url(site);
            easy.post(true).unwrap();
            easy.post_field_size(data.len() as u64).unwrap();
            let mut header = List::new();
            header.append("Content-Type: application/json");
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
                .status().unwrap();

            ui.set_textBox("".to_shared_string());

            //weak.unwrap().window().hide().unwrap();

        }
    });
    
    ui.run()?;
    Ok(())
}
