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
use midir::{Ignore, MidiInput};
use std::io::{Write, stdin, stdout};
use std::sync::mpsc::channel; // For robust port selection
// ‼️ --- END NEW IMPORTS ---

fn main() -> Result<(), Box<dyn error::Error>> {
    // ‼️ --- MIDI Setup ---
    // We use a channel to send MIDI messages from the (separate) MIDI
    // callback thread to our main loop.
    let (tx, rx) = channel();
    let mut midi_in = MidiInput::new("push2_input_demo")?;
    midi_in.ignore(Ignore::None);

    // --- Robust Port Selection ---
    // This code will list all MIDI inputs and let you pick the right one.
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
            print!("Please select port for Ableton Push 2: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            let port_index: usize = input.trim().parse()?;
            in_ports
                .get(port_index)
                .ok_or("Invalid port index selected")?
        }
    };

    let port_name = midi_in.port_name(in_port)?;
    println!("\nOpening connection to: {}", port_name);

    // --- Connect to MIDI port ---
    // The `_conn` variable must stay in scope for the connection to be kept alive.
    // The callback (the closure) runs on its own thread.
    let _conn = midi_in.connect(
        in_port,
        "push2-input-connection",
        move |_stamp, message, _| {
            // Send the raw MIDI message bytes to the main thread
            if let Err(e) = tx.send(message.to_vec()) {
                println!("Error sending MIDI message from callback: {}", e);
            }
        },
        (), // No user data needed
    )?;
    // ‼️ --- End MIDI Setup ---

    // --- Original Display Setup ---
    let mut display = Push2Display::new()?;
    let text_style = MonoTextStyle::new(&FONT_10X20, Bgr565::WHITE);
    let mut position = Point::new(0, 70);
    let mut step = 4;

    println!("Connection open. Press any button on your Push 2...");
    println!("The raw MIDI bytes will be printed below.");

    loop {
        // ‼️ --- Check for MIDI messages ---
        // We use try_recv to be non-blocking, so it doesn't stop our animation.
        // Use a `while let` to process all messages received since the last frame.
        while let Ok(message) = rx.try_recv() {
            // message is a Vec<u8>
            //
            // A common pad PRESS (Note On) will look like:
            // [144, 36, 127]
            //  |    |    `-- Velocity (how hard it was pressed)
            //  |    `------- Note Number (36-99 for the 64 pads)
            //  `----------- 144 = Note On (Channel 1)
            //
            // A pad RELEASE (Note Off) will look like:
            // [128, 36, 0] or [144, 36, 0]
            //

            // println!("MIDI message: {:?}", message);

            // Example: Check for a pad press (Note On event)
            if message.len() > 2 && message[0] == 144 && message[2] > 0 {
                println!("--- Pad {} PRESSED --- {}", message[1], message[2]);
                // You could change the text, color, or position here!
            }
        }
        // ‼️ --- End Check for MIDI messages ---

        // --- Original Display Logic ---
        display.clear(Bgr565::BLACK)?;

        Rectangle::new(Point::zero(), display.size())
            .into_styled(PrimitiveStyle::with_stroke(Bgr565::WHITE, 1))
            .draw(&mut display)?;

        position.x += step;
        if position.x >= display.size().width as i32 || position.x <= 0 {
            step *= -1;
        }

        Text::new("Hello!", position, text_style).draw(&mut display)?;

        display.flush()?; // if no frame arrives in 2 seconds, the display is turned black
        thread::sleep(time::Duration::from_millis(1000 / 60));
    }
}
