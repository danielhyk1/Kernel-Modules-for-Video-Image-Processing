
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{Read, Write, Error};

mod utils;
use utils::*;
use tflitec::interpreter::{Interpreter, Options};

fn handle_client(mut stream: TcpStream) -> Result<(), Error> {
	//TODO Translate 1d-> YUV422->RGB->MATRIX
	println!("address: {}", stream.peer_addr()?);
	// load model and create interpreter
	let options = Options::default();
	let path = format!("resource/lite-model_movenet_singlepose_lightning_tflite_int8_4.tflite");
	let interpreter = Interpreter::with_model_path(&path, Some(options)).unwrap();
	interpreter.allocate_tensors().expect("Allocate tensors [FAILED]");
	
	let mut buf = [0; 110592];
	loop{
		let read = stream.read(&mut buf)?;
		if read == 0 { return Ok(()) }
		let vec_1d = buf.to_vec();

		// set input (tensor0)
		interpreter.copy(&vec_1d[..], 0).unwrap();
			
		// run interpreter
		interpreter.invoke().expect("Invoke [FAILED]");

		// get output
		let output_tensor = interpreter.output(0).unwrap();
		stream.write(&output_tensor.data());
	}
}

fn main() {
	//Set Listener
	let listener = TcpListener::bind("127.0.0.1:8888").expect("could not bind");
	for stream in listener.incoming() {
		match stream{
			Err(e) => { eprintln!("failed: {}", e) }
			Ok(stream) => {
				thread::spawn(move || {
					handle_client(stream).unwrap_or_else(|error| eprintln!("{:?}", error));
				});
			}
		}
	}
}
