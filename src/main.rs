use std::net::*;
use std::io::prelude::*;
use std::mem::*;
use std::vec::Vec;

/*
TODO:
1. разобраться с модулями
2. реализовать оставшиеся операции
3. демо-приложение
*/

#[repr(C)]
enum E {
    I(i32),
    D(f64),
    S(String),
    None,
}

impl E {
	fn println(&self) {
		match self {
			&E::I(ref i) => println!("Int: {}", i),
			&E::D(ref d) => println!("Double: {}", d),
			&E::S(ref s) => println!("String: {}", s),
			&E::None => println!("Wildcard"),
		}
	}
}

struct LinuxTuplesConnection {
	connection: SocketAddr,
}


const PUT: i32 = 0;
const GET: i32 = 1;
const READ: i32 = 2;
const GET_NB: i32 = 3;
const READ_NB: i32 = 4;
const DUMP: i32 = 5;
const COUNT: i32 = 6;
const REPLACE: i32 = 8;
/*
const LOG: i32 = 7;
*/

const asciiS: i32 = 115;
const asciiI: i32 = 105;
const asciiD: i32 = 100;
const asciiQ: i32 = 63;
const asciiT: i32 = 116;

fn ti(i: i32) -> [u8; 4] {
	let buff: [u8; 4];
	unsafe {
		buff = transmute(i);
	}
	return buff;
} 

fn ctoi(arr: &mut [u8; 24], mut val: i32, offset: isize) {
	unsafe {
		std::ptr::copy_nonoverlapping(&mut val as *mut _ as *mut u8, arr.as_ptr().offset(offset) as *mut u8, std::mem::size_of::<i32>());
	}
}

fn ciot(arr:&[u8; 24], offset:isize) -> i32 {
	let mut val: i32 = 0;
	
	unsafe {
		std::ptr::copy_nonoverlapping(arr.as_ptr().offset(offset) as *mut u8, &mut val as *mut _ as *mut u8, std::mem::size_of::<i32>());
	}
	
	return val;
}

fn cdot(arr:&[u8; 24], offset:isize) -> f64 {
	let mut val: f64 = 0.;
	unsafe {
		std::ptr::copy_nonoverlapping(arr.as_ptr().offset(offset) as *mut u8, &mut val as *mut _ as *mut u8, std::mem::size_of::<f64>());
	}
	return val;
}

fn ctod(arr: &mut [u8; 24], mut val: f64, offset: isize) {
	unsafe {
		std::ptr::copy_nonoverlapping(&mut val as *mut _ as *mut u8, arr.as_ptr().offset(offset) as *mut u8, std::mem::size_of::<f64>());
	}
}  

fn send_tuple(tuple: &Vec<E>, stream: &mut TcpStream) {
	
    stream.write_all(& (ti(tuple.len() as i32)));
    
    let mut string_length:i32 = 0;
    
    for elem in tuple {
    	match elem {
    		&E::S(ref s) => string_length += s.len() as i32,
    		_ => {}
    	}
    }
    
    stream.write_all(&ti(string_length));
    
    string_length = 0;
    
    let mut buff: [u8; 24] = [0; 24];
    
    for elem in tuple {
    	match elem {
    		&E::I(ref i) => {
				//tag
   	 			ctoi(&mut buff, asciiI, 0);
   	 			//union
   	 			ctoi(&mut buff, *i, 8);
   	 			
   	 			stream.write_all(&buff); 
    		},
    		&E::D(ref d) => {
    			//tag
   	 			ctoi(&mut buff, asciiD, 0);
   	 			//union
   	 			ctod(&mut buff, *d, 8);
   	 			stream.write_all(&buff);
   	 			
    		},
    		&E::S(ref s) => {
    			//tag
   	 			ctoi(&mut buff, asciiS, 0);
   	 			
   	 			//union
   	 			ctoi(&mut buff, string_length, 8);
   	 			ctoi(&mut buff, s.len() as i32, 16);
   	 			
   	 			//tuple pointer
   	 			stream.write_all(&buff);
   	 			
   	 			string_length += s.len() as i32;
    		}
    		&E::None => {
    			ctoi(&mut buff, asciiQ, 0);
    			
    			stream.write_all(&buff);
    		}
    	}
    }
    
    for elem in tuple {
    	match elem {
    		&E::S(ref s) => {
    			stream.write_all(s.as_bytes());
    		}
    		_ => {}
    	}
    }
}

fn recv_tuple(stream: &mut TcpStream) -> Vec<E> {
	
	struct str_desc {
		used: bool,
		offset: i32,
		len: i32,
	}
	
	let mut tuple: Vec<E> = Vec::<E>::new();
	let mut str_descs: Vec<str_desc> = Vec::<str_desc>::new();
	
	let mut buff_4:[u8; 4] = [0; 4];
	let mut buff_8:[u8; 8] = [0; 8];
	let mut buff:[u8; 24] = [0; 24];
	
	stream.read_exact(&mut buff_4);
	let num_elements:i32 = unsafe { transmute(buff_4) };
	
	//non-blocking reads/writes
	if num_elements == -1 {
		return tuple;
	}
	
	stream.read_exact(&mut buff_4);
	let string_length:i32 = unsafe { transmute(buff_4) };
	
	for i in 0..num_elements {
		
		stream.read_exact(&mut buff);
		
		let tag: i32 = ciot(&buff, 0);
		
		match tag {
			asciiI => {
				tuple.push(E::I(ciot(&buff, 8)));
				str_descs.push(str_desc { used: false, offset: 0, len: 0} );
			},
			asciiD => {
				tuple.push(E::D(cdot(&buff, 8)));
				str_descs.push(str_desc { used: false, offset: 0, len: 0} );
			},
			asciiS => {
				str_descs.push(str_desc { 
						used: true,
						offset: ciot(&buff, 8),
						len: ciot(&buff, 16),
					});
				tuple.push(E::S("".to_string()));
			},
			asciiQ => {
				str_descs.push(str_desc { used: false, offset: 0, len: 0} );
			},
			_ => {
				
			}
		}
	}
	
	if string_length != 0 {
		let mut string_space:Vec<u8> = vec![0; string_length as usize];
		stream.read_exact(&mut string_space);
		
		let mut str_slice = string_space.as_slice();
		for i in 0..num_elements {
			if str_descs[i as usize].used == true {
				tuple[i as usize] = E::S(String::from_utf8(
					string_space[
						(str_descs[i as usize].offset as usize)..
						(str_descs[i as usize].offset + str_descs[i as usize].len) as usize].to_vec()).unwrap());
			}
		}
		
	}
	
	return tuple;
}

impl LinuxTuplesConnection {
	fn put_tuple(&self, tuple: &Vec<E>) {
	    let mut stream_err = TcpStream::connect(&self.connection);
	    match stream_err {
	    	Ok(mut stream) => {
	    		stream.write_all(&ti(PUT));
				send_tuple(tuple, &mut stream);
				stream.shutdown(Shutdown::Write);
	    	}
	    	Err(why) => {
	    		println!("{}", why);
	    	}
	    }
	}
	
	fn get_tuple(&self, tuple: &Vec<E>) -> Vec<E>
	{
		let mut stream_err = TcpStream::connect(&self.connection);
		match stream_err {
			Ok(mut stream) => {
				stream.write_all(&ti(GET));
				
				send_tuple(tuple, &mut stream);
				
				stream.shutdown(Shutdown::Write);
				
				return recv_tuple(&mut stream);
			}
			Err(why) => {
				println!("{}", why);
				return Vec::<E>::new();
			}
		}
	}
	
	fn read_tuple(&self, tuple: &Vec<E>) -> Vec<E>
	{
		let mut stream_err = TcpStream::connect(&self.connection);
		match stream_err {
			Ok(mut stream) => {
				stream.write_all(&ti(READ));
				
				send_tuple(tuple, &mut stream);
				
				stream.shutdown(Shutdown::Write);
				
				return recv_tuple(&mut stream);
			}
			Err(why) => {
				println!("{}", why);
				return Vec::<E>::new();
			}
		}
	}
	
	fn get_nb_tuple(&self, tuple: &Vec<E>) -> Vec<E>
	{
		let mut stream_err = TcpStream::connect(&self.connection);
		match stream_err {
			Ok(mut stream) => {
				stream.write_all(&ti(GET_NB));
				
				send_tuple(tuple, &mut stream);
				
				stream.shutdown(Shutdown::Write);
				
				return recv_tuple(&mut stream);
			}
			Err(why) => {
				println!("{}", why);
				return Vec::<E>::new();
			}
		}
	}
	
	fn read_nb_tuple(&self, tuple: &Vec<E>) -> Vec<E>
	{
		let mut stream_err = TcpStream::connect(&self.connection);
		match stream_err {
			Ok(mut stream) => {
				stream.write_all(&ti(READ_NB));
				
				send_tuple(tuple, &mut stream);
				
				stream.shutdown(Shutdown::Write);
				
				return recv_tuple(&mut stream);
			}
			Err(why) => {
				println!("{}", why);
				return Vec::<E>::new();
			}
		}
	}
	
	fn read_all_tuples(&self, tuples: &Vec<Vec<E>>) -> Vec<Vec<E>>
	{
		let mut stream_err = TcpStream::connect(&self.connection);
		match stream_err {
			Ok(mut stream) => {
				stream.write_all(&ti(DUMP));
				let count: i32 = tuples.len() as i32;
				stream.write_all(&ti(count));
				
				for tuple in tuples {
					send_tuple(tuple, &mut stream);
				}
				
				let mut buff_4:[u8; 4] = [0; 4];
				
				stream.read_exact(&mut buff_4);
				
				let recv_count = unsafe { transmute(buff_4) };
				
				let mut result:Vec<Vec<E>> = Vec::<Vec<E>>::new();
				
				for i in 0..recv_count {
					result.push(recv_tuple(&mut stream));
				}
				return result;
			}
			Err(why) => {
				println!("{}", why);
				return Vec::<Vec<E>>::new();
			}
		}
	}
	
	fn number_of_tuples(&self, tuples: &Vec<Vec<E>>) -> i32
	{
		let mut stream_err = TcpStream::connect(&self.connection);
		match stream_err {
			Ok(mut stream) => {
				stream.write_all(&ti(COUNT));
				let count: i32 = tuples.len() as i32;
				stream.write_all(&ti(count));
				
				for tuple in tuples {
					send_tuple(tuple, &mut stream);
				}
				
				let mut buff_4:[u8; 4] = [0; 4];
				
				stream.read_exact(&mut buff_4);
				
				let recv_count = unsafe { transmute(buff_4) };
				
				return recv_count;
			}
			Err(why) => {
				println!("{}", why);
				return -1;
			}
		}
	}
	
	fn replace_tuple(&self, tuple: &Vec<E>, replacement: &Vec<E>) {
			let mut stream_err = TcpStream::connect(&self.connection);
			match stream_err {
				Ok(mut stream) => {
					stream.write_all(&ti(REPLACE));
					send_tuple(tuple, &mut stream);
					send_tuple(replacement, &mut stream);
					let mut buff_4:[u8; 4] = [0; 4];
					stream.read_exact(&mut buff_4);
					
					let op:i32 = unsafe { transmute(buff_4) };
					if op != REPLACE {
						println!("Replacement error");
					}
				}
				Err(why) => {
					println!("{}", why);
				}
			}
	}
}


fn main() {
    let conn: SocketAddr = ("127.0.0.1:5000").parse().unwrap();
    let tuple = vec![E::I(10), E::D(10.1), E::S("123".to_string())];
    let tuple2 = vec![E::S("12789".to_string()), E::I(10), E::D(5.), E::S("1234".to_string())];
    let tuple3 = vec![E::S("12abv".to_string()), E::I(11), E::D(5.), E::S("123".to_string())];
    
    let serv = LinuxTuplesConnection {connection : conn};
    
    serv.put_tuple(&tuple);
    serv.put_tuple(&tuple2);
    serv.put_tuple(&tuple3);
    
    let templates: Vec<Vec<E>> = vec![];
	let tuples_dump = serv.read_all_tuples(&templates);
	
	for tuple in tuples_dump {
		for item in tuple {
			item.println();
		}
		println!("");
	} 
	
	println!("Count: {}", serv.number_of_tuples(&templates));
	
	serv.replace_tuple(&tuple2, &tuple3);
	
	let tuples_dump = serv.read_all_tuples(&templates);
	
	for tuple in tuples_dump {
		for item in tuple {
			item.println();
		}
		println!("");
	} 
	
	println!("Count: {}", serv.number_of_tuples(&templates));
	
}
