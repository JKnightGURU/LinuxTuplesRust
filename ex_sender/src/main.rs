use std::net::*;
use std::vec::Vec;

extern crate linux_tuples_client;
extern crate rand;

use linux_tuples_client::*;


fn main() {
	if std::env::args().count() != 6
	{
		println!("ex_sender. USAGE: ex_sender <IP> <port> <node name> <command> <cmd_text>");
		return;
	}
	
	let ip: String = std::env::args().nth(1).unwrap();
	let port: String = std::env::args().nth(2).unwrap();
	let name: String = std::env::args().nth(3).unwrap();
	let command: String = std::env::args().nth(4).unwrap();
	let text: String = std::env::args().nth(5).unwrap();
	
	let conn: SocketAddr = (format!("{}:{}", ip, port)).parse().unwrap();
	let serv = LinuxTuplesConnection { connection: conn };
	
	let cmd_id: i32 = rand::random::<i32>();
	
	let command_tuple = vec![E::S(name.clone()), E::S(command), E::S(text), E::I(cmd_id)];
	
	serv.put_tuple(&command_tuple);	
	
	let recv_tuple = vec![E::I(cmd_id), E::None];
	
	let output = serv.get_tuple(&recv_tuple).unwrap();
	
	match &output[1] {
		&E::S(ref s) => {
			println!("{}",s);
			
		}
		_ => {}
	}
	
	println!("Done!");
}
