use serde::Deserialize;
use std::collections::HashSet;
use std::env;
use std::process::{Command, exit};

#[derive(Deserialize)]
struct Room {
    room_id: String,
}

#[derive(Deserialize)]
struct Member {
    user_id: String,
}

fn main() {
    // Get username from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} [--debug] <matrix_username>", args[0]);
        exit(1);
    }
    
    // Check for --debug flag
    let debug = args.contains(&"--debug".to_string());
    
    // Get username (skip program name and --debug flag if present)
    let username = args.iter()
        .skip(1)
        .find(|arg| *arg != "--debug")
        .expect("Matrix username required");

    // Get user's rooms using synadm
    let rooms = get_user_rooms(username, debug);
    if rooms.is_empty() {
        println!("No rooms found for user {}", username);
        return;
    }

    // Collect all contacts from all rooms
    let mut all_contacts: HashSet<String> = HashSet::new();

    for room in rooms {
        let members = get_room_members(&room.room_id, debug);
        for member in members {
            // Add all members except the queried user
            if member.user_id != *username {
                all_contacts.insert(member.user_id);
            }
        }
    }

    // Display results
    println!("Contacts for user {}:", username);
    let mut sorted_contacts: Vec<String> = all_contacts.into_iter().collect();
    sorted_contacts.sort();
    for contact in sorted_contacts {
        println!("{}", contact);
    }
}

fn get_user_rooms(username: &str, debug: bool) -> Vec<Room> {
    let args = ["user", "rooms", username];
    
    if debug {
        eprintln!("DEBUG: synadm {}", args.join(" "));
    }
    
    let output = Command::new("synadm")
        .args(args)
        .output()
        .expect("Failed to execute synadm user rooms command");

    if !output.status.success() {
        eprintln!(
            "Error executing synadm user rooms: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        exit(1);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).expect("Failed to parse JSON from synadm user rooms")
}

fn get_room_members(room_id: &str, debug: bool) -> Vec<Member> {
    let args = ["room", "members", room_id];
    
    if debug {
        eprintln!("DEBUG: synadm {}", args.join(" "));
    }
    
    let output = Command::new("synadm")
        .args(args)
        .output()
        .expect("Failed to execute synadm room members command");

    if !output.status.success() {
        eprintln!(
            "Error executing synadm room members for {}: {}",
            room_id,
            String::from_utf8_lossy(&output.stderr)
        );
        return Vec::new();
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).unwrap_or_else(|e| {
        eprintln!(
            "Failed to parse JSON from synadm room members for {}: {}",
            room_id, e
        );
        Vec::new()
    })
}
