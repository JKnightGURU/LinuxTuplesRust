use std::net::*;
use std::vec::Vec;

extern crate linux_tuples_client;
extern crate time;

use linux_tuples_client::*;


fn main() {
	if std::env::args().count() < 4 
	{
		println!("ex_sender. USAGE: ex_sender <IP> <port> <command> [<cmd_text> <node name>]");
		return;
	}
	
	let ip: String = std::env::args().nth(1).unwrap();
	let port: String = std::env::args().nth(2).unwrap();
	let mut name: String = String::default();
	let command: String = std::env::args().nth(3).unwrap();
	let mut text: String = String::default(); 
	
	if (command != "users")
	{
		text = std::env::args().nth(4).unwrap();
		if (command != "global") {
			name = std::env::args().nth(5).unwrap();
		}
	}
	
	let conn: SocketAddr = (format!("{}:{}", ip, port)).parse().unwrap();
	let serv = LinuxTuplesConnection { connection: conn };
	
	if command != "users" {
		use time;
		let cmd_id: i32 = time::now().tm_sec;
		if command != "global" {
			serv.put_tuple(&vec![E::S(name.clone()), E::S(command), E::S(text), E::I(cmd_id)]);	
			let output = serv.get_tuple(&vec![E::I(cmd_id), E::None]).unwrap();
			
			match &output[1] {
				&E::S(ref s) => {
					println!("{}",s);
					
				}
				_ => {}
			}
			
			println!("Done!");
		} else {
			if let E::I(count) = serv.read_tuple(&vec![E::S("USERS_COUNT".to_string()), E::None]).unwrap()[1] {
				serv.put_tuple(&vec![E::I(cmd_id.clone()), E::S(command.clone()), E::S(text.clone())]);
				
				for i in 0..count {
					let output = serv.get_tuple(&vec![E::None, E::I(cmd_id), E::None]).unwrap();
					if let &E::S(ref name) = &output[0] {
						println!("Name: {}", name);
					}
					if let &E::S(ref out) = &output[2] {
						println!("{}", out);
					}
					println!("");
				}
				
				serv.get_tuple(&vec![E::I(cmd_id), E::S(command.clone()), E::S(text.clone())]);
				
				println!("Done!");
			}
			
		}
	}
	else
	{
		let users = serv.read_nb_tuple(&vec![E::S("USER_LIST".to_string()), E::None]).unwrap();
		if users.len() > 0 {
			match &users[1] {
				&E::T(ref t) => {
					for user in t {
						match user {
							&E::S(ref username) => println!("{}", username),
							_ => {}
						}
					}	 
				}
				_ => {}
			}
		}
	}
}
