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
		if let E::I(count) = users_count[1] {
				let users_count_updated = vec![E::S("USERS_COUNT".to_string()), E::I(count + 1)];
				serv.put_tuple(&users_count_updated);
		}
		
		let mut users_tuple = serv.get_tuple(&users_list_request_tuple).unwrap();
		if let &mut E::T(ref mut v) = &mut users_tuple[1] {
				v.push(E::S(name.clone()));
				let users_list_confirm_tuple = vec![E::S("USER_LIST".to_string()), E::T(v.clone())];
				serv.put_tuple(&users_list_confirm_tuple);
		}
	} else {
		let users_count_init_tuple = vec![E::S("USERS_COUNT".to_string()), E::I(1)];
		serv.put_tuple(&users_count_init_tuple);
		let users_list_confirm_tuple = vec![E::S("USER_LIST".to_string()), E::T(vec![E::S(name.clone())])];
		serv.put_tuple(&users_list_confirm_tuple);
	}
	
	let serv_global = serv.clone();
	let name_global = name.clone();
	
	let th = std::thread::spawn(move || {
			let mut cmd_id: i32 = 0;
			
			'thread: loop {
				let command_received = serv_global.read_tuple(&vec![E::None, E::S("global".to_string()), E::None]).unwrap();
				
				let mut text:String = "".to_string();
				
				if let E::I(id) = command_received[0] {
					if cmd_id == id {
						continue 'thread;
					}
					cmd_id = id;
				}
				
				if let &E::S(ref txt) = &command_received[2] {
					text = txt.clone();
				}
				
				let mut sp = text.split(" ").collect::<Vec<&str>>();
				let mut process = std::process::Command::new(sp[0]);
				for i in 1..sp.len()
				{
					process.arg(sp[i]);
				}
				let output = String::from_utf8(process.output().unwrap().stdout).unwrap();
				
				
				serv_global.put_tuple(&vec![E::S(name_global.clone()), E::I(cmd_id), E::S(output)]);
			} 
	});
	
	//name-based thread
	'main: loop {
		let command_received = serv.get_tuple(
			&vec![E::S(name.clone()), E::None, E::None, E::None]).unwrap();
		
		//LinuxTuplesConnection::print_tuple(&command_received);
		
		if let &E::S(ref s) = &command_received[2] {
				let mut sp = s.split(" ").collect::<Vec<&str>>();
				let mut process = std::process::Command::new(sp[0]);
				for i in 1..sp.len()
				{
					process.arg(sp[i]);
				}
				let output = process.output().unwrap().stdout;
				
				if let &E::I(ref command_id) = &command_received[3] {
						let output_tuple = vec![E::I(*command_id), E::S(String::from_utf8(output).unwrap())];
						serv.put_tuple(&output_tuple);
				}
				//println!("{}", String::from_utf8(output).unwrap());
		}
		
		println!("");
		if command_received[1] == E::S("shutdown".to_string())
		{
			let users_count = serv.get_tuple(&users_count_request_tuple).unwrap();
			if let E::I(count) = users_count[1] {
				if count > 1 {
					serv.put_tuple(&vec![E::S("USERS_COUNT".to_string()), E::I(count - 1)]);
				}
			}
			
			let mut users_list = serv.get_tuple(&users_list_request_tuple).unwrap();
			let mut shouldRemove: bool = false;
			
			
			if let &mut E::T(ref mut users) = &mut users_list[1] {
				let mut ind: usize = 0;
				for i in 0..users.len() {
					if users[i] == E::S(name.clone()) {
								ind = i;
					}
				}
				users.remove(ind);
				if users.len() == 0 {
					shouldRemove = true;
				}							
			}
					
			if !shouldRemove {
				serv.put_tuple(&users_list);
			} 
					
			break 'main;
		}
	}

}
