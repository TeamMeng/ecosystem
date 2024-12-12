use std::{thread, time::Duration};
use sysinfo::System;

fn main() {
    let mut system = System::new_all();

    println!("total cpu use: {:.2}", system.global_cpu_usage());
    println!("total memory: {} KB", system.total_memory());
    println!("already use memory: {} KB\n", system.used_memory());

    let high_cpu_process = system
        .processes()
        .values()
        .max_by(|a, b| a.cpu_usage().partial_cmp(&b.cpu_usage()).unwrap());

    if let Some(process) = high_cpu_process {
        println!(
            "high cpu process's name: {:?}, PID: {}, cpu use: {}, memory: {}\n",
            process.name(),
            process.pid(),
            process.cpu_usage(),
            process.memory()
        );
    }

    loop {
        system.refresh_all();

        let target_process = system
            .processes()
            .values()
            .find(|process| process.name().to_ascii_lowercase() == "google chrome");

        match target_process {
            Some(process) => {
                println!("process: {:?}", process.name());
                println!("PID: {}", process.pid());
                println!("CPU use: {:.2}", process.cpu_usage());
                println!("memory: {}", process.memory());
                println!("status: {:?}\n", process.status());
            }
            None => println!("not found"),
        }

        thread::sleep(Duration::from_secs(1));
    }
}
