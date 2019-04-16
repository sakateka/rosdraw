#[macro_use]
extern crate log;

use env_logger;
use ipc_channel::ipc::{IpcOneShotServer, IpcSender};
use std::io::{self, BufRead};

use std::{env, process};

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("RUST_LOG", env::var("RUST_LOG").unwrap_or("info".into()));
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        error!("Usage: {} <number of circles>", args[0]);
        process::exit(1);
    }
    let num_circles = args[1]
        .parse::<usize>()
        .map_err(|e| {
            error!("Can't parse number of circles from '{}': {:?}", args[1], e);
            process::exit(1);
        })
        .unwrap();

    info!("Create ipc channel");
    let (server, server_name) = IpcOneShotServer::new().unwrap();

    info!("Launch draw programm");
    let mut child = process::Command::new("./lab-draw")
        .arg(server_name)
        .spawn()
        .expect("Failed to execute command");
    if let Ok(exit) = child.try_wait() {
        error!("Exit status from child {:?}", exit);
    }

    let (_, tx): (_, IpcSender<(usize, f32)>) = server.accept().unwrap();
    info!("Send num circles: {}", num_circles);
    match tx.send((num_circles, 10.0)) {
        Ok(_) => {
            for line in io::stdin().lock().lines() {
                let line = line.unwrap();
                let cmd: Vec<&str> = line.split_whitespace().take(2).collect();
                let idx = cmd.get(0);
                let speed = cmd.get(1);
                if idx.is_none() || speed.is_none() {
                    error!("Incorrect control command, format: <idx as usize> <speed as f32>");
                    continue;
                }
                let idx = idx.unwrap().parse::<usize>();
                let speed = speed.unwrap().parse::<f32>();
                if idx.is_err() || speed.is_err() {
                    error!("Failed to parse idx={:?}, speed={:?}", idx, speed);
                    continue;
                }
                let idx = idx.unwrap();
                let speed = speed.unwrap();
                info!("Send speed={} to circle idx={}", speed, idx);
                match tx.send((idx, speed)) {
                    Err(e) => error!("Failed to send circle={} speed={} {:?}", idx, speed, e),
                    Ok(_) => (),
                }
            }
        }
        Err(e) => error!("Failed to send num circles {:?}", e),
    }

    let _ = child.kill();
    child.wait().unwrap();
}
