// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    ui.set_rectWidth(30.0);
    ui.set_rectX(235.0);
    ui.on_editedText({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_rectWidth(30.0 + (30.0 * ui.get_textBox().len() as f32));
            ui.set_rectX(235.0 - (15.0 * ui.get_textBox().len() as f32));
        }
    });
    let weak = ui.as_weak();
    ui.on_closeWindow({
        move || {
            weak.unwrap().window().hide().unwrap();
        }
    });
    
    ui.run()?;
    Ok(())
}
