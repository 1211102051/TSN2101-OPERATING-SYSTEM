#![allow(unused_imports)]
use inquire::{InquireError, Select, Text};
use tabled::{Table, Tabled, settings::{Alignment, Disable, object::{Columns, Rows}, Rotate, style::{LineText, Style}}};

#[derive(Clone, Debug, Tabled)]
struct Process {
    id: String,
    arrival_time: i32,
    burst_time: i32,
    priority: i32,
    completion_time: i32,
    turnaround_time: i32,
    waiting_time: i32
}

fn print_gantt(gantt: &Vec<String>, complete: &mut Vec<i32>) {
    let mut row_pos: usize = 0;
    let mut table = Table::new(gantt);
    let gantt_arrival: String;
    match complete.first() {
        Some(value) => {
            gantt_arrival = value.to_string();
            complete.remove(0);
        },
        None => panic!("No arrival time.")
    }
    table.with(Disable::row(Rows::first())).with(Rotate::Left).with(Style::modern());
    table.with(LineText::new(gantt_arrival, Rows::last()));
    for (_i, item) in complete.iter().enumerate() {
        row_pos += 1;
        table.with(LineText::new(item.to_string(), Rows::last()).offset((row_pos * 5) - item.to_string().len() + 1));
    }
    table.to_string();
    println!("\n{}", table);
}

fn calc_avg_times(processes: &Vec<Process>) {
    let mut table = Table::new(processes);
    table.with(Style::rounded());
    table.with(Disable::column(Columns::single(3)));
    table.to_string();
    println!("\n{}", table);
    let total_turnaround_time: i32 = processes.iter().map(|p| p.turnaround_time).sum();
    let total_waiting_time: i32 = processes.iter().map(|p| p.waiting_time).sum();
    let number_of_processes: i32 = processes.len() as i32;
    println!("Average Turnaround Time: {} / {} = {}", total_turnaround_time, number_of_processes, total_turnaround_time as f64 / number_of_processes as f64);
    println!("Average Waiting Time: {} / {} = {}", total_waiting_time, number_of_processes, total_waiting_time as f64 / number_of_processes as f64);
}

#[allow(dead_code)]
fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

fn update_process_times(processes: &mut Vec<Process>, id: &str, completion_time: i32) {
    if let Some(process) = processes.iter_mut().find(|p| p.id == id) {
        process.completion_time = completion_time;
        process.turnaround_time = completion_time - process.arrival_time;
        process.waiting_time = process.turnaround_time - process.burst_time;
    } else {
        println!("Process with ID {} not found.", id);
    }
}

fn round_robin(processes: &mut Vec<Process>, quantum: i32) {
    let mut processes_copy: Vec<Process> = processes.clone();
    processes.sort_by_key(|process| (process.arrival_time, process.priority));
    let mut time: i32 = 0;
    let mut shift_queue: bool = false;
    let mut queue: Vec<Process> = Vec::new();
    let mut gantt: Vec<String> = Vec::new();
    let mut complete: Vec<i32> = Vec::new();
    complete.push(processes.get(0).unwrap().arrival_time);
    while !processes.is_empty() || !queue.is_empty() {
        // Add processes to the queue if their arrival time is less than or equal to the current time
        for process in processes.iter().filter(|p| p.arrival_time <= time) {
            queue.push(process.clone());
        }
        processes.retain(|p| p.arrival_time > time);

        if shift_queue == false {
            if !queue.is_empty() {
                shift_queue = true;
            }
        } else {
            queue.rotate_left(1);
        }

        if queue.is_empty() {
            time += 1;
            continue;
        } else {
            let mut process = queue.remove(0);
            let pid = process.id.clone();
            let p_bt = process.burst_time;

            if p_bt <= quantum {
                time += p_bt;
                gantt.push(pid.clone());
                complete.push(time);
                shift_queue = false;
                update_process_times(&mut processes_copy, &pid, time);
            } else {
                time += quantum;
                process.burst_time -= quantum;
                queue.insert(0, process);
                gantt.push(pid);
                complete.push(time);
            }
        }
    }

    print_gantt(&gantt, &mut complete);
    calc_avg_times(&processes_copy);
}

fn non_preemptive_sjf(processes: &mut Vec<Process>) {
    let mut processes_copy: Vec<Process> = processes.clone();
    processes.sort_by(|a, b| a.arrival_time.cmp(&b.arrival_time));
    let mut time: i32 = 0;
    let mut complete: Vec<i32> = Vec::new();
    let mut gantt: Vec<String> = Vec::new();
    let mut queue: Vec<Process> = Vec::new();

    complete.push(processes.get(0).unwrap().arrival_time);
    
    while !processes.is_empty() {
        queue.clear();
        for process in processes.iter() {
            let at: i32 = process.arrival_time;
            if at <= time {
                queue.push(process.clone());
            }
        }
        if !queue.is_empty() {
            queue.sort_by(|a, b| a.burst_time.cmp(&b.burst_time));
            let process: Process = queue.get(0).unwrap().clone();
            let pid: String = process.id;
            time += process.burst_time;
            gantt.push(pid.clone());
            complete.push(time);
            update_process_times(&mut processes_copy, &pid, time);
            let index: usize = processes.iter().position(|x| x.id == pid).unwrap();
            processes.remove(index);
        } else {
            time += 1;
            continue;
        }
    }
    print_gantt(&gantt, &mut complete);
    calc_avg_times(&processes_copy);
}

fn preemptive_sjf(processes: &mut Vec<Process>) {
    let mut processes_copy: Vec<Process> = processes.clone();
    processes.sort_by_key(|processes| (processes.arrival_time, processes.priority));
    let mut time: i32 = 0;
    let mut complete: Vec<i32> = Vec::new();
    let mut gantt: Vec<String> = Vec::new();
    let mut queue: Vec<Process> = Vec::new();
    let mut old_pid: String = processes.get(0).unwrap().id.clone();
    let total_time: i32 = processes.iter().fold(0, |a, b| a + b.burst_time);
    complete.push(processes.get(0).unwrap().arrival_time);
    for _i in 0..total_time {
        if !processes.is_empty() {
            for process in processes.iter() {
                let at: i32 = process.arrival_time;
                if at <= time {
                    queue.push(process.clone());
                }
            }
            for process in queue.iter() {
                processes.retain(|x| x.id != process.id);
            }
        }
        if queue.is_empty() {
            time += 1;
            continue;
        }
        queue.sort_by_key(|processes| (processes.burst_time, processes.arrival_time));
        let mut process: Process = queue.drain(0..1).next().unwrap().clone();
        if old_pid != process.id.clone() {
            gantt.push(old_pid);
            complete.push(time);
            old_pid = process.id.clone();
        }
        if queue.is_empty() && processes.is_empty() {
            gantt.push(process.id.clone());
            complete.push(time+process.burst_time.clone());
            update_process_times(&mut processes_copy, &process.id, time+process.burst_time.clone());
            break;
        }
        process.burst_time -= 1;
        time += 1;
        if process.burst_time == 0 {
            //add complete time
            update_process_times(&mut processes_copy, &old_pid, time);
        } else {
            queue.push(process);
        }
    }
    print_gantt(&gantt, &mut complete);
    calc_avg_times(&processes_copy);
}

fn non_preemptive_priority(processes: &mut Vec<Process>) {
    let mut processes_copy: Vec<Process> = processes.clone();
    processes.sort_by(|a, b| a.arrival_time.cmp(&b.arrival_time));
    let mut time: i32 = 0;
    let mut complete: Vec<i32> = Vec::new();
    let mut gantt: Vec<String> = Vec::new();
    let mut queue: Vec<Process> = Vec::new();

    complete.push(processes.get(0).unwrap().arrival_time);

    while !processes.is_empty() {
        queue.clear();
        for process in processes.iter() {
            let at: i32 = process.arrival_time;
            if at <= time {
                queue.push(process.clone());
            }
        }
        if !queue.is_empty() {
            queue.sort_by(|a, b| a.priority.cmp(&b.priority));
            let process: Process = queue.get(0).unwrap().clone();
            let pid: String = process.id;
            time += process.burst_time;
            gantt.push(pid.clone());
            complete.push(time);
            update_process_times(&mut processes_copy, &pid, time);
            let index: usize = processes.iter().position(|x| x.id == pid).unwrap();
            processes.remove(index);
        } else {
            time += 1;
            continue;
        }
    }
    print_gantt(&gantt, &mut complete);
    calc_avg_times(&processes_copy);
}

fn preemptive_priority(processes: &mut Vec<Process>) {
    let mut processes_copy: Vec<Process> = processes.clone();
    processes.sort_by_key(|processes| (processes.arrival_time, processes.priority));
    let mut time: i32 = 0;
    let mut complete: Vec<i32> = Vec::new();
    let mut gantt: Vec<String> = Vec::new();
    let mut queue: Vec<Process> = Vec::new();
    let mut old_pid: String = processes.get(0).unwrap().id.clone();
    let total_time: i32 = processes.iter().fold(0, |a, b| a + b.burst_time);
    complete.push(processes.get(0).unwrap().arrival_time);
    for _i in 0..total_time {
        if !processes.is_empty() {
            for process in processes.iter() {
                let at: i32 = process.arrival_time;
                if at <= time {
                    queue.push(process.clone());
                }
            }
            for process in queue.iter() {
                processes.retain(|x| x.id != process.id);
            }
        }
        if queue.is_empty() {
            time += 1;
            continue;
        }
        queue.sort_by_key(|processes| (processes.priority, processes.arrival_time));
        let mut process: Process = queue.drain(0..1).next().unwrap().clone();
        if old_pid != process.id.clone() {
            gantt.push(old_pid);
            complete.push(time);
            old_pid = process.id.clone();
        }
        if queue.is_empty() && processes.is_empty() {
            gantt.push(process.id.clone());
            complete.push(time+process.burst_time.clone());
                update_process_times(&mut processes_copy, &process.id, time+process.burst_time);
            break;
        }
        process.burst_time -= 1;
        time += 1;
        if process.burst_time == 0 {
            //add complete time
            update_process_times(&mut processes_copy, &old_pid, time);
        } else {
            queue.push(process);
        }
    }
    print_gantt(&gantt, &mut complete);
    calc_avg_times(&processes_copy);
}

fn main() {
    clearscreen::clear().expect("Failed to clear screen.");
    let menu_options: Vec<&str> = vec!["Create New Processes", "Use Existing Processes", "Exit"];
    let algo_options: Vec<&str> = vec!["Round Robin", "Shortest Job First(Preemptive)", "Shortest Job First(Non-Preemptive)", "Priority(Preemptive)", "Priority(Non-Preemptive)"];
    let mut processes: Vec<Process> = Vec::new();

    let menu_selection: Result<&str, InquireError> = Select::new("Choose an option", menu_options).prompt();

    match menu_selection {
        Ok(index) => {
            match index {
                "Create New Processes" => {
                    let number_of_processes: i32;
                    let entered_no_p = Text::new("Enter number of processes => ").prompt();
                    match entered_no_p {
                        Ok(entered_no_p) => number_of_processes = entered_no_p.parse::<i32>().unwrap(),
                        Err(_) => panic!("No process entered."),
                    }
                    for i in 0..number_of_processes {
                        let id: String = format!("P{}", i);
                        let arrival_time: i32;
                        let burst_time: i32;
                        let priority: i32;
                        println!("\nEntering process {} details", i);
                        let entered_a_t: Result<String, InquireError> = Text::new("Enter arrival time => ").prompt();
                        match entered_a_t {
                            Ok(entered_a_t) => arrival_time = entered_a_t.parse::<i32>().unwrap(),
                            Err(_) => panic!("No arrival time entered.")
                        }
                        let entered_b_t: Result<String, InquireError> = Text::new("Enter burst time => ").prompt();
                        match entered_b_t {
                            Ok(entered_b_t) => burst_time = entered_b_t.parse::<i32>().unwrap(),
                            Err(_) => panic!("No burst time entered.")
                        }
                        let entered_p: Result<String, InquireError> = Text::new("Enter priority => ").prompt();
                        match entered_p {
                            Ok(entered_p) => priority = entered_p.parse::<i32>().unwrap(),
                            Err(_) => panic!("No priority entered.")
                        }
                        processes.push(Process{id: id, arrival_time: arrival_time, burst_time: burst_time, priority: priority, completion_time: 0, turnaround_time: 0, waiting_time: 0});
                    }
                },
                "Use Existing Processes" => {
                    println!("Initializing Processes...");
                    processes.push(Process{id: "P0".to_string(), arrival_time: 0, burst_time: 6, priority: 3, completion_time: 0, turnaround_time: 0, waiting_time: 0});
                    processes.push(Process{id: "P1".to_string(), arrival_time: 1, burst_time: 4, priority: 3, completion_time: 0, turnaround_time: 0, waiting_time: 0});
                    processes.push(Process{id: "P2".to_string(), arrival_time: 5, burst_time: 6, priority: 1, completion_time: 0, turnaround_time: 0, waiting_time: 0});
                    processes.push(Process{id: "P3".to_string(), arrival_time: 6, burst_time: 6, priority: 1, completion_time: 0, turnaround_time: 0, waiting_time: 0});
                    processes.push(Process{id: "P4".to_string(), arrival_time: 7, burst_time: 6, priority: 5, completion_time: 0, turnaround_time: 0, waiting_time: 0});
                    processes.push(Process{id: "P5".to_string(), arrival_time: 8, burst_time: 6, priority: 6, completion_time: 0, turnaround_time: 0, waiting_time: 0});
                },
                "Exit" => std::process::exit(1),
                _ => panic!("No option selected!")
            }
        }
        Err(_) => {println!("You didn't choose anything.");}
    }

    clearscreen::clear().expect("Failed to clear screen.");
    let mut table = Table::new(processes.clone());
    table.with(Style::rounded());
    table.with(Disable::column(Columns::single(4)));
    table.with(Disable::column(Columns::single(5)));
    table.with(Disable::column(Columns::last()));
    table.to_string();
    println!("\n{}", table);

    let algo_selection: Result<&str, InquireError> = Select::new("Choose an algorithm", algo_options).prompt();

    match algo_selection {
        Ok(algorithm) => {
            match algorithm {
                "Round Robin" => {
                    let time = Text::new("Enter time quanta =>").prompt();
                    match time {
                        Ok(time) => round_robin(&mut processes, time.parse::<i32>().unwrap()),
                        Err(_) => panic!("No quanta entered.")
                    }
                },
                "Shortest Job First(Preemptive)" => preemptive_sjf(&mut processes),
                "Shortest Job First(Non-Preemptive)" => non_preemptive_sjf(&mut processes),
                "Priority(Preemptive)" => preemptive_priority(&mut processes),
                "Priority(Non-Preemptive)" => non_preemptive_priority(&mut processes),
                _ => panic!("No algorithm selected.")
            }
        }
        Err(_) => println!("There was an error, please try again")
    }
}
