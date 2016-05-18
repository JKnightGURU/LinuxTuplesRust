use std::net::*;
use std::vec::Vec;

extern crate linux_tuples_client;
use linux_tuples_client::*;


fn main() {
	if std::env::args().count() != 4
	{
		println!("ex_listener. USAGE: ex_listener <IP> <port> <node name>");
		return;
	}
	
	let ip: String = std::env::args().nth(1).unwrap();
	let port: String = std::env::args().nth(2).unwrap();
	let name: String = std::env::args().nth(3).unwrap();
	
	let conn: SocketAddr = (format!("{}:{}", ip, port)).parse().unwrap();
	let serv = LinuxTuplesConnection { connection: conn };
	
	let users_count_request_tuple: Vec<E> = vec![E::S("USERS_COUNT".to_string()), E::None];
	let users_list_request_tuple: Vec<E> = vec![E::S("USER_LIST".to_string()), E::None];
	
	let users_count_list = serv.read_nb_tuple(&users_count_request_tuple).unwrap();
	if users_count_list.len() > 0
	{
		let users_count = serv.get_tuple(&users_count_request_tuple).unwrap();
		match users_count[0] {
			E::I(count) => {
				let users_count_updated = vec![E::S("USERS_COUNT".to_string()), E::I(count + 1)];
				serv.put_tuple(&users_count_updated);
			}
			_ => {}
		}
		
		let mut users_tuple = serv.get_tuple(&users_list_request_tuple).unwrap();
		let ref mut users = users_tuple[1];
		match users {
			&mut E::T(ref mut v) => {
				v.push(E::S(name.clone()));
				let users_list_confirm_tuple = vec![E::S("USER_LIST".to_string()), E::T(v.clone())];
				serv.put_tuple(&users_list_confirm_tuple);
			}
			_ => {}
		}
	} else {
		let users_count_init_tuple = vec![E::S("USERS_COUNT".to_string()), E::I(1)];
		serv.put_tuple(&users_count_init_tuple);
		let users_list_confirm_tuple = vec![E::S("USER_LIST".to_string()), E::T(vec![E::S(name.clone())])];
		serv.put_tuple(&users_list_confirm_tuple);
	}
	
	//name-based thread
	'main: loop {
		let expect_command_tuple = vec![E::S(name.clone()), E::None, E::None, E::None];
		let command_received = serv.get_tuple(&expect_command_tuple).unwrap();
		
		//LinuxTuplesConnection::print_tuple(&command_received);
		
		match &command_received[2] {
			&E::S(ref s) => {
				let mut sp = s.split(" ").collect::<Vec<&str>>();
				let mut process = std::process::Command::new(sp[0]);
				for i in 1..sp.len()
				{
					process.arg(sp[i]);
				}
				let output = process.output().unwrap().stdout;
				
				match &command_received[3] {
					&E::I(ref command_id) => {
						let output_tuple = vec![E::I(*command_id), E::S(String::from_utf8(output).unwrap())];
						serv.put_tuple(&output_tuple);
					}
					_ => {}
				}
				//println!("{}", String::from_utf8(output).unwrap());
			}
			_ => {}
		}
		
		println!("");
		match &command_received[1] {
			&E::S(ref s) => {
				if s.as_str() == "shutdown"
				{
					let users_count = serv.get_tuple(&users_count_request_tuple).unwrap();
					match users_count[0] {
						E::I(count) => {
							let users_count_updated = vec![E::S("USERS_COUNT".to_string()), E::I(count + 1)];
							serv.put_tuple(&users_count_updated);
						}
						_ => {}
					}
					break 'main;
				}
			}
			_ => {}
		}
	}
}
