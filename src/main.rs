use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_10X20},
    pixelcolor::Bgr565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use push2_display::Push2Display;
use std::{error, thread, time};

// ‼️ --- NEW IMPORTS ---
use midir::{Ignore, MidiInput, MidiOutput}; // Added MidiOutput
use std::io::{Write, stdin, stdout};
use std::sync::mpsc::channel;
// ‼️ --- END NEW IMPORTS ---

fn main() -> Result<(), Box<dyn error::Error>> {
    // --- 1. MIDI Input Setup (Same as before) ---
    let (tx, rx) = channel();
    let mut midi_in = MidiInput::new("push2_input_demo")?;
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

    // ‼️ --- 2. MIDI Output Setup (NEW!) ---
    let midi_out = MidiOutput::new("push2_output_demo")?;

    // --- Port Selection for Output ---
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

    // Connect to the output port.
    // We get a `conn_out` object that we can use to send messages.
    let mut conn_out = midi_out.connect(out_port, "push2-output-connection")?;
    // ‼️ --- End MIDI Output Setup ---

    // --- 3. Original Display Setup ---
    let mut display = Push2Display::new()?;
    let text_style = MonoTextStyle::new(&FONT_10X20, Bgr565::WHITE);
    let mut position = Point::new(0, 70);
    let mut step = 4;

    println!("\nConnection open. Press any pad...");

    loop {
        // --- 4. Check for MIDI messages (Modified!) ---
        while let Ok(message) = rx.try_recv() {
            if message.len() < 3 {
                continue;
            }

            let status = message[0];
            let note = message[1];
            let velocity = message[2];

            match status {
                // 144 = "Note On"
                144 => {
                    if velocity > 0 {
                        println!("--- Pad {} PRESSED (vel {}) ---", note, velocity);

                        // ‼️ SEND MESSAGE TO LIGHT UP PAD
                        // We send a "Note On" message back to the *same note*.
                        // The "velocity" of the *output* message determines the color.
                        // 122 is a bright white. Try other values!
                        let color_message = &[144, note, 122];
                        conn_out.send(color_message)?;
                    } else {
                        // "Note On" with velocity 0 is a "Note Off"
                        println!("--- Pad {} RELEASED ---", note);

                        // ‼️ SEND MESSAGE TO TURN OFF PAD
                        // A "Note Off" (128) or "Note On" with 0 velocity works.
                        let off_message = &[128, note, 0];
                        conn_out.send(off_message)?;
                    }
                }

                // 128 = "Note Off"
                128 => {
                    println!("--- Pad {} RELEASED ---", note);

                    // ‼️ SEND MESSAGE TO TURN OFF PAD
                    let off_message = &[128, note, 0];
                    conn_out.send(off_message)?;
                }

                // 176 = "Control Change" (knobs, buttons above screen)
                176 => {
                    if velocity == 127 {
                        // 127 = button press
                        println!("--- Button {} PRESSED ---", note);
                        // ‼️ SEND MESSAGE TO LIGHT UP BUTTON
                        // Control Change buttons also use "Note On" for their lights.
                        // Note numbers for buttons are different (e.g., 20-29, 102-117)
                        let on_message = &[144, note, 127]; // 127 = bright white
                        conn_out.send(on_message)?;
                    } else {
                        // 0 = button release
                        println!("--- Button {} RELEASED ---", note);
                        // ‼️ SEND MESSAGE TO TURN OFF BUTTON
                        let off_message = &[144, note, 0]; // 0 = off
                        conn_out.send(off_message)?;
                    }
                }

                _ => {} // Ignore aftertouch, etc. for now
            }
        }

        // --- 5. Original Display Logic (Unchanged) ---
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
