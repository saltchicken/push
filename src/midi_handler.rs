use crate::app_config::AppConfig;
use midir::{
    Ignore, MidiInput, MidiInputConnection, MidiInputPort, MidiOutput, MidiOutputConnection,
    MidiOutputPort,
};
use std::error::Error;
use std::io::{Write, stdin, stdout};
use std::sync::mpsc::Sender;

/// Holds the MIDI connections.
/// `_conn_in` is kept to ensure it stays alive (RAII).
/// `conn_out` is public so `main.rs` can send messages.
pub struct MidiHandler {
    _conn_in: MidiInputConnection<()>,
    pub conn_out: MidiOutputConnection,
}

impl MidiHandler {
    /// Creates a new MidiHandler, finds and connects to ports.
    pub fn new(config: &AppConfig, tx: Sender<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        // --- Input Connection ---
        let mut midi_in = MidiInput::new("push2")?;
        midi_in.ignore(Ignore::None);

        let in_port = Self::select_input_port(&midi_in, &config.midi_input_port)?;

        let in_port_name = midi_in.port_name(&in_port)?;
        println!("Opening input connection to: {}", in_port_name);

        let _conn_in = midi_in.connect(
            &in_port,
            "push2-input-connection",
            move |_stamp, message, _| {
                tx.send(message.to_vec()).unwrap();
            },
            (),
        )?;

        // --- Output Connection ---
        let midi_out = MidiOutput::new("push2_output")?;

        let out_port = Self::select_output_port(&midi_out, &config.midi_output_port)?;

        let out_port_name = midi_out.port_name(&out_port)?;
        println!("Opening output connection to: {}", out_port_name);

        let conn_out = midi_out.connect(&out_port, "push2-output-connection")?;

        Ok(MidiHandler { _conn_in, conn_out })
    }

    /// Finds the configured input port, or falls back to manual selection.
    fn select_input_port(
        midi_in: &MidiInput,
        config_port_name: &str,
    ) -> Result<MidiInputPort, Box<dyn Error>> {
        let in_ports = midi_in.ports();

        // Try to find port from config
        for port in &in_ports {
            if midi_in.port_name(port)? == config_port_name {
                println!("Found configured input port: {}", config_port_name);
                return Ok(port.clone());
            }
        }

        // Configured port not found, fall back to old logic
        println!(
            "WARN: Configured input port '{}' not found. Falling back to manual selection.",
            config_port_name
        );
        match in_ports.len() {
            0 => Err("No MIDI input ports found!".into()),
            1 => {
                println!(
                    "Choosing the only available input port: {}",
                    midi_in.port_name(&in_ports[0])?
                );
                Ok(in_ports[0].clone())
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
                in_ports
                    .get(port_index)
                    .cloned()
                    .ok_or("Invalid input port index".into())
            }
        }
    }

    /// Finds the configured output port, or falls back to manual selection.
    fn select_output_port(
        midi_out: &MidiOutput,
        config_port_name: &str,
    ) -> Result<MidiOutputPort, Box<dyn Error>> {
        let out_ports = midi_out.ports();

        // Try to find output port from config
        for port in &out_ports {
            if midi_out.port_name(port)? == config_port_name {
                println!("Found configured output port: {}", config_port_name);
                return Ok(port.clone());
            }
        }

        // Configured port not found, fall back to old logic
        println!(
            "WARN: Configured output port '{}' not found. Falling back to manual selection.",
            config_port_name
        );
        match out_ports.len() {
            0 => Err("No MIDI output ports found!".into()),
            1 => {
                println!(
                    "Choosing the only available output port: {}",
                    midi_out.port_name(&out_ports[0])?
                );
                Ok(out_ports[0].clone())
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
                    .cloned()
                    .ok_or("Invalid output port index".into())
            }
        }
    }
}

