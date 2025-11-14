use serde::Deserialize;
use std::collections::HashSet;
use std::env;
use std::process::{Command, exit};

#[derive(Deserialize)]
struct Room {
    room_id: String,
}

#[derive(Deserialize)]
struct UserMembershipResponse {
    joined_rooms: Vec<String>,
    total: usize,
}

#[derive(Deserialize)]
struct Member {
    user_id: String,
}

#[derive(Deserialize)]
struct RoomMembersResponse {
    members: Vec<String>,
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
    let args = ["user", "membership", username];
    
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
    if debug {
        eprintln!("DEBUG: synadm output: {}", stdout);
    }
    
    let response: UserMembershipResponse = serde_json::from_str(&stdout)
        .expect("Failed to parse JSON from synadm user rooms");
    
    // Convert room IDs from strings to Room structs
    response.joined_rooms
        .into_iter()
        .map(|room_id| Room { room_id })
        .collect()
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
    
    // Parse the response as RoomMembersResponse and convert to Vec<Member>
    let response: RoomMembersResponse = match serde_json::from_str(&stdout) {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "Failed to parse JSON from synadm room members for {}: {}",
                room_id, e
            );
            return Vec::new();
        }
    };
    
    // Convert Vec<String> to Vec<Member>
    response.members
        .into_iter()
        .map(|user_id| Member { user_id })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_membership_response_deserialization() {
        let json = r#"{
            "joined_rooms": [
                "!DEOhdyEateKRHcRuJX:matrix.fdn.fr",
                "!xqCsWJjzgamGppeBiy:matrix.fdn.fr",
                "!YmaFJxbsfKeXAjKiVl:matrix.fdn.fr",
                "!uNTFtzaiShUjyOsILl:matrix.fdn.fr",
                "!amKyxBfzQfKtaCLRrL:matrix.fdn.fr",
                "!DNBNEtOjLJqAGdAlqx:matrix.fdn.fr",
                "!SGMazuMtZjkjOedPTQ:matrix.fdn.fr"
            ],
            "total": 7
        }"#;

        let response: UserMembershipResponse = serde_json::from_str(json)
            .expect("Failed to deserialize UserMembershipResponse");

        assert_eq!(response.joined_rooms.len(), 7);
        assert_eq!(response.total, 7);
        assert_eq!(
            response.joined_rooms[0],
            "!DEOhdyEateKRHcRuJX:matrix.fdn.fr"
        );
    }

    #[test]
    fn test_room_id_conversion() {
        let json = r#"{
            "joined_rooms": [
                "!room1:example.com",
                "!room2:example.com"
            ],
            "total": 2
        }"#;

        let response: UserMembershipResponse = serde_json::from_str(json)
            .expect("Failed to deserialize UserMembershipResponse");

        let rooms: Vec<Room> = response.joined_rooms
            .into_iter()
            .map(|room_id| Room { room_id })
            .collect();

        assert_eq!(rooms.len(), 2);
        assert_eq!(rooms[0].room_id, "!room1:example.com");
        assert_eq!(rooms[1].room_id, "!room2:example.com");
    }

    #[test]
    fn test_empty_membership_response() {
        let json = r#"{
            "joined_rooms": [],
            "total": 0
        }"#;

        let response: UserMembershipResponse = serde_json::from_str(json)
            .expect("Failed to deserialize UserMembershipResponse");

        assert_eq!(response.joined_rooms.len(), 0);
        assert_eq!(response.total, 0);
    }

    #[test]
    fn test_room_members_response_deserialization() {
        let json = r#"{
            "members": [
                "@user1:example.com",
                "@user2:example.com",
                "@user3:example.com"
            ]
        }"#;

        let response: RoomMembersResponse = serde_json::from_str(json)
            .expect("Failed to deserialize RoomMembersResponse");

        assert_eq!(response.members.len(), 3);
        assert_eq!(response.members[0], "@user1:example.com");
        assert_eq!(response.members[1], "@user2:example.com");
        assert_eq!(response.members[2], "@user3:example.com");
    }

    #[test]
    fn test_empty_room_members_response() {
        let json = r#"{
            "members": []
        }"#;

        let response: RoomMembersResponse = serde_json::from_str(json)
            .expect("Failed to deserialize RoomMembersResponse");

        assert_eq!(response.members.len(), 0);
    }

    #[test]
    fn test_room_members_to_member_conversion() {
        let json = r#"{
            "members": [
                "@alice:example.com",
                "@bob:example.com"
            ]
        }"#;

        let response: RoomMembersResponse = serde_json::from_str(json)
            .expect("Failed to deserialize RoomMembersResponse");

        let members: Vec<Member> = response.members
            .into_iter()
            .map(|user_id| Member { user_id })
            .collect();

        assert_eq!(members.len(), 2);
        assert_eq!(members[0].user_id, "@alice:example.com");
        assert_eq!(members[1].user_id, "@bob:example.com");
    }
}
