extern crate sysinfo;
use sysinfo::{ProcessExt, SystemExt};

use std::process::Command;
use std::env;
use std::path::PathBuf;
use std::fs;
use std::thread;
use std::time;
use std::process::exit;

fn is_already_running() -> bool {
    let self_pid = sysinfo::get_current_pid();
    let mut system = sysinfo::System::new();
    system.refresh_processes();

    for (pid, process) in system.get_process_list() {
        let name = process.name();
        if name.contains("roseline") && self_pid != *pid {
            println!("Found Roseline's process '{}' with pid={}. Exiting...", name, pid);
            return true;
        }
    }

    false
}

fn get_dirs() -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let mut current_dir = env::current_exe().unwrap();
    current_dir.pop();

    let mut roseline_log = current_dir.clone();
    let mut roseline_web_log = current_dir.clone();
    roseline_log.push("roseline.log");
    roseline_web_log.push("roseline-web.log");

    let mut roseline_exe = current_dir;
    let mut roseline_web_exe = roseline_exe.clone();
    if cfg!(windows) {
        roseline_exe.push("roseline.exe");
        roseline_web_exe.push("roseline-web.exe");
    } else {
        roseline_exe.push("roseline");
        roseline_web_exe.push("roseline-web");
    };

    (roseline_log, roseline_exe, roseline_web_log, roseline_web_exe)
}

fn open_log(path: &PathBuf) -> fs::File {
    match fs::OpenOptions::new().create(true).write(true).truncate(true).open(&path) {
        Ok(file) => file,
        Err(error) => {
            eprintln!("{}: Unable to open log file. Error: {}", path.display(), error);
            exit(1);
        }
    }
}

fn main() {
    if is_already_running() {
        return;
    }

    const TIMEOUT_MS: u64 = 10000;

    let (roseline_log, roseline_exe, roseline_web_log, roseline_web_exe) = get_dirs();
    println!("Roseline={}\nLog={}", roseline_exe.display(), roseline_log.display());
    println!("Roseline-web={}\nLog={}", roseline_web_exe.display(), roseline_web_log.display());

    if !roseline_exe.exists() {
        eprintln!("Roseline exe is missing: {}", roseline_exe.display());
        return;
    }
    else if !roseline_web_exe.exists() {
        eprintln!("Roseline-web exe is missing: {}", roseline_exe.display());
        return;
    }

    let roseline_log = open_log(&roseline_log);
    let roseline_web_log = open_log(&roseline_web_log);

    thread::spawn(move || {
        loop {
            let stdout = roseline_log.try_clone().expect("Cannot clone log file");
            let stderr = roseline_log.try_clone().expect("Cannot clone log file");
            match Command::new(&roseline_exe).stdout(stdout).stderr(stderr).status() {
                Ok(status) => {
                    match status.success() {
                        true => println!("Roseline successfully finished"),
                        false => {
                            println!("Roseline finished with errors");
                            thread::sleep(time::Duration::from_millis(TIMEOUT_MS));
                        }
                    }
                },
                Err(error) => {
                    eprintln!("Couldn't run Roseline. Error: {}", error);
                    return;
                }
            }
        }
    });

    loop {
        let stdout = roseline_web_log.try_clone().expect("Cannot clone log file");
        let stderr = roseline_web_log.try_clone().expect("Cannot clone log file");
        match Command::new(&roseline_web_exe).stdout(stdout).stderr(stderr).status() {
            Ok(status) => {
                match status.success() {
                    true => println!("Roseline-web successfully finished"),
                    false => {
                        println!("Roseline-web finished with errors");
                        thread::sleep(time::Duration::from_millis(TIMEOUT_MS));
                    }
                }
            },
            Err(error) => {
                eprintln!("Couldn't run Roseline-web. Error: {}", error);
                return;
            }
        }
    }
}
