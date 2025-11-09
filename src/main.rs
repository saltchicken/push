mod button_map;
use button_map::ButtonMap;

use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_10X20},
    pixelcolor::Bgr565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use midir::{Ignore, MidiInput, MidiOutput};
use push2_display::Push2Display;
use std::io::{Write, stdin, stdout};
use std::sync::mpsc::channel;
use std::{error, thread, time};

fn main() -> Result<(), Box<dyn error::Error>> {
    // --- MIDI Setup  ---
    let (tx, rx) = channel();
    let mut midi_in = MidiInput::new("push2")?;
    midi_in.ignore(Ignore::None);
    let in_ports = midi_in.ports();
    let in_port = match in_ports.len() {
        0 => return Err("No MIDI input ports found!".into()),
        1 => {
            println!(
                "Choosing the only available input port: {}",
                midi_in.port_name(&in_ports[0])?
            );
            &in_ports[0]
        }
        _ => {
            println!("\nAvailable input ports:");
            for (i, port) in in_ports.iter().enumerate() {
                println!("{}: {}", i, midi_in.port_name(port)?);
            }
            print!("Please select port for Ableton Push 2 INPUT: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            let port_index: usize = input.trim().parse()?;
            in_ports.get(port_index).ok_or("Invalid input port index")?
        }
    };
    let in_port_name = midi_in.port_name(in_port)?;
    println!("Opening input connection to: {}", in_port_name);
    let _conn_in = midi_in.connect(
        in_port,
        "push2-input-connection",
        move |_stamp, message, _| {
            tx.send(message.to_vec()).unwrap();
        },
        (),
    )?;
    let midi_out = MidiOutput::new("push2_output_demo")?;
    let out_ports = midi_out.ports();
    let out_port = match out_ports.len() {
        0 => return Err("No MIDI output ports found!".into()),
        1 => {
            println!(
                "Choosing the only available output port: {}",
                midi_out.port_name(&out_ports[0])?
            );
            &out_ports[0]
        }
        _ => {
            println!("\nAvailable output ports:");
            for (i, port) in out_ports.iter().enumerate() {
                println!("{}: {}", i, midi_out.port_name(port)?);
            }
            print!("Please select port for Ableton Push 2 OUTPUT: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            let port_index: usize = input.trim().parse()?;
            out_ports
                .get(port_index)
                .ok_or("Invalid output port index")?
        }
    };
    let out_port_name = midi_out.port_name(out_port)?;
    println!("Opening output connection to: {}", out_port_name);
    let mut conn_out = midi_out.connect(out_port, "push2-output-connection")?;

    // --- Display Setup ---
    let mut display = Push2Display::new()?;
    let text_style = MonoTextStyle::new(&FONT_10X20, Bgr565::WHITE);
    let mut position = Point::new(0, 70);
    let mut step = 4;

    // --- Create ButtonMap ---
    let button_map = ButtonMap::new()?;
    println!("\nConnection open. Press any pad...");

    // --- Main Loop ---
    loop {
        while let Ok(message) = rx.try_recv() {
            if message.len() < 3 {
                continue;
            }

            let status = message[0];
            let address = message[1];
            let velocity = message[2];

            println!(
                "Received message: status: {}, address: {}, velocity: {}",
                status, address, velocity
            );

            // We match on the STATUS byte first!
            match status {
                // --- NOTE ON / NOTE OFF (144 or 128) ---
                144 | 128 => {
                    // This is a pad, so we check the note_map
                    if let Some(pad_coord) = button_map.get_note(address) {
                        if status == 144 && velocity > 0 {
                            // Note On
                            println!("--- Pad ({}, {}) PRESSED ---", pad_coord.x, pad_coord.y);
                            conn_out.send(&[144, address, 122])?; // 122 = White
                        } else {
                            // Note Off (128 or 144 w/ vel 0)
                            println!("--- Pad ({}, {}) RELEASED ---", pad_coord.x, pad_coord.y);
                            conn_out.send(&[128, address, 0])?; // 0 = Off
                        }
                    }
                }

                // --- CONTROL CHANGE (176) ---
                176 => {
                    // This could be a button OR an encoder, so we check both maps
                    if let Some(control_name) = button_map.get_control(address) {
                        if velocity > 0 {
                            // Button down
                            println!("--- Button {:?} PRESSED ---", control_name);
                            // conn_out.send(&[144, address, 127])?; // 127 = Bright White
                        } else {
                            // Button up
                            println!("--- Button {:?} RELEASED ---", control_name);
                            // conn_out.send(&[144, address, 0])?; // 0 = Off
                        }
                    } else if let Some(encoder_name) = button_map.get_encoder(address) {
                        println!(
                            "--- Encoder {:?} TWISTED, value {} ---",
                            encoder_name, velocity
                        );
                    }
                }
                _ => {} // Ignore other messages
            }
        }

        // --- Original Display Logic ---
        // TODO: Replace this with something that is not a test case
        display.clear(Bgr565::BLACK)?;
        Rectangle::new(Point::zero(), display.size())
            .into_styled(PrimitiveStyle::with_stroke(Bgr565::WHITE, 1))
            .draw(&mut display)?;
        position.x += step;
        if position.x >= display.size().width as i32 || position.x <= 0 {
            step *= -1;
        }
        Text::new("Hello!", position, text_style).draw(&mut display)?;
        display.flush()?;
        thread::sleep(time::Duration::from_millis(1000 / 60));
    }
}
