// ‼️ NEW FILE (examples/slint_example.rs)
use push2::{Push2, Push2Event}; // From your existing library
use std::{error::Error, thread, time::Duration};

// --- Slint-specific imports ---
use slint;
slint::include_modules!(); // This macro loads the UI from `build.rs`

// --- We can re-use display drawing code from push_example.rs ---
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_10X20},
    pixelcolor::Bgr565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};

fn main() -> Result<(), Box<dyn Error>> {
    // --- 1. Initialize the Slint window on the main thread ---
    let window = HelloWorld::new().unwrap();

    // Get a weak handle to pass to the other thread
    let window_weak = window.as_weak();

    // --- 2. Spawn the Push 2 logic in a separate thread ---
    println!("Spawning Push 2 handler thread...");
    thread::spawn(move || {
        // --- This is the logic from the original push_example.rs ---
        let mut push2 = Push2::new().expect("Failed to initialize Push 2");

        let text_style = MonoTextStyle::new(&FONT_10X20, Bgr565::WHITE);
        let mut position = Point::new(0, 70);
        let mut step = 4;
        println!("\nPush 2 Thread: Connection open. Press any pad...");

        // --- This is the main loop for the Push 2 thread ---
        loop {
            // --- 3. Poll for Push 2 events ---
            while let Some(event) = push2.poll_event() {
                println!("Push 2 Thread received event: {:?}", event);

                // --- 4. Send the event to the Slint GUI thread ---
                let event_text = format!("{:?}", event);

                // ‼️ Clone the weak pointer (cheap)
                let window_weak_clone = window_weak.clone();

                // ‼️ Use invoke_from_event_loop for thread-safe GUI updates
                // ‼️ We MOVE the weak pointer and the text into the closure
                let _ = slint::invoke_from_event_loop(move || {
                    // ‼️ This closure runs on the GUI thread
                    // ‼️ NOW it is safe to upgrade the handle
                    if let Some(window) = window_weak_clone.upgrade() {
                        window.invoke_push2_event_received(event_text.into());
                    }
                });
            }

            // --- Original Display Logic (runs on this thread) ---
            push2.display.clear(Bgr565::BLACK).unwrap();
            Rectangle::new(Point::zero(), push2.display.size())
                .into_styled(PrimitiveStyle::with_stroke(Bgr565::WHITE, 1))
                .draw(&mut push2.display)
                .unwrap();

            position.x += step;
            if position.x >= push2.display.size().width as i32 || position.x <= 0 {
                step *= -1;
            }
            Text::new("Hello!", position, text_style)
                .draw(&mut push2.display)
                .unwrap();

            push2.display.flush().unwrap();

            // Sleep to maintain ~60fps for the Push 2 display
            thread::sleep(Duration::from_millis(1000 / 60));
        }
    });

    // --- 5. Run the Slint event loop (blocks the main thread) ---
    println!("Starting Slint GUI event loop on main thread...");
    window.run().unwrap();

    Ok(())
}
