use std::net::*;
use std::io::prelude::*;
use std::mem::*;
use std::vec::Vec;

#[repr(C)]
enum E {
    Int(i32),
    Double(f64),
    Str(String),
    None,
}

impl E {
	fn println(&self) {
		match self {
			&E::Int(ref i) => println!("Int: {}", i),
			&E::Double(ref d) => println!("Double: {}", d),
			&E::Str(ref s) => println!("String: {}", s),
			&E::None => println!("Wildcard"),
		}
	}
}



const PUT: i32 = 0;
const GET: i32 = 1;
const READ: i32 = 2;
/*
const GET_NB: i32 = 3;
const READ_NB: i32 = 4;
const DUMP: i32 = 5;
const COUNT: i32 = 6;
const LOG: i32 = 7;
const REPLACE: i32 = 8;
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
    		&E::Str(ref s) => string_length += s.len() as i32,
    		_ => {}
    	}
    }
    
    stream.write_all(&ti(string_length));
    
    string_length = 0;
    
    let mut buff: [u8; 24] = [0; 24];
    
    for elem in tuple {
    	match elem {
    		&E::Int(ref i) => {
				//tag
   	 			ctoi(&mut buff, asciiI, 0);
   	 			//union
   	 			ctoi(&mut buff, *i, 8);
   	 			
   	 			stream.write_all(&buff); 
    		},
    		&E::Double(ref d) => {
    			//tag
   	 			ctoi(&mut buff, asciiD, 0);
   	 			//union
   	 			ctod(&mut buff, *d, 8);
   	 			stream.write_all(&buff);
   	 			
    		},
    		&E::Str(ref s) => {
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
    		&E::Str(ref s) => {
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
				tuple.push(E::Int(ciot(&buff, 8)));
				str_descs.push(str_desc { used: false, offset: 0, len: 0} );
			},
			asciiD => {
				tuple.push(E::Double(cdot(&buff, 8)));
				str_descs.push(str_desc { used: false, offset: 0, len: 0} );
			},
			asciiS => {
				str_descs.push(str_desc { 
						used: true,
						offset: ciot(&buff, 8),
						len: ciot(&buff, 16),
					});
				tuple.push(E::Str("".to_string()));
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
				tuple[i as usize] = E::Str(String::from_utf8(
					string_space[
						(str_descs[i as usize].offset as usize)..
						(str_descs[i as usize].offset + str_descs[i as usize].len) as usize].to_vec()).unwrap());
			}
		}
		
	}
	
	return tuple;
}

fn put_tuple(tuple: &Vec<E>, connection: &SocketAddr) {
    let mut stream_err = TcpStream::connect(connection);
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

fn get_tuple(tuple: &Vec<E>, connection: &SocketAddr) -> Vec<E>
{
	let mut stream_err = TcpStream::connect(connection);
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

fn read_tuple(tuple: &Vec<E>, connection: &SocketAddr) -> Vec<E>
{
	let mut stream_err = TcpStream::connect(connection);
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

fn main() {
    let connection: SocketAddr = ("127.0.0.1:5000").parse().unwrap();
    let tuple = vec![E::Int(10), E::Double(10.1), E::Str("123".to_string())];
    let tuple2 = vec![E::Str("12789".to_string()), E::Int(10), E::Double(5.), E::Str("1234".to_string())];
    let tuple3 = vec![E::Str("12abv".to_string()), E::Int(11), E::Double(5.), E::Str("123".to_string())];
    
    put_tuple(&tuple, &connection);
    put_tuple(&tuple2, &connection);
    put_tuple(&tuple3, &connection);
    
    let tuple4 = read_tuple(&vec![E::None, E::Double(10.1), E::None], &connection);
    
    for val in tuple4 {
    	val.println();
    }
}
