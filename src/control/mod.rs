use std::io::{Read};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use termion::async_stdin;
use termion::raw::IntoRawMode;

#[derive(Debug, Clone)]
pub enum MicState {
    Record,
    Stop, 
}

pub fn setup_control() -> Arc<Mutex<MicState>> {
    let state: Arc<Mutex<MicState>> = Arc::new(Mutex::new(MicState::Stop));

    let state_clone = Arc::clone(&state);
    let mut stdin = async_stdin().bytes();
    thread::spawn(move || loop {
        let b = stdin.next();
        match b {
            Some(Ok(b'r')) => {
                *state.lock().unwrap() = MicState::Record;
            }
            Some(Ok(b'q')) => {
                *state.lock().unwrap() = MicState::Stop;
            }
            //Some(Ok(b)) => {
                //*state.lock().unwrap() = b.to_string();
            //}
            _ => {}
        };
        //thread::sleep(std::time::Duration::from_millis(25));
    });
    
    Arc::clone(&state_clone)
}
