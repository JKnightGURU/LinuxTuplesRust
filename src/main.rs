use std::net::*;
use std::io::prelude::*;
use std::mem::*;

#[repr(C)]
enum E {
    Int(i32),
    Double(f64),
    Str(String),
    None,
}



const PUT: i32 = 0;
/*
const GET: i32 = 1;
const READ: i32 = 2;
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

fn ctoi(arr: &mut [u8; 24], mut val: i32, offset: isize)
{
	unsafe {
		std::ptr::copy_nonoverlapping(&mut val as *mut _ as *mut u8, arr.as_ptr().offset(offset) as *mut u8, std::mem::size_of::<i32>());
	}
}  

fn ctod(arr: &mut [u8; 24], mut val: f64, offset: isize)
{
	unsafe {
		std::ptr::copy_nonoverlapping(&mut val as *mut _ as *mut u8, arr.as_ptr().offset(offset) as *mut u8, std::mem::size_of::<f64>());
	}
}  

fn send_tuple(tuple: &[E], stream: &mut TcpStream) {
	
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

fn put_tuple(tuple: &[E], connection: &SocketAddr) {
    let mut stream = TcpStream::connect(connection).unwrap();
    
	stream.write_all(&ti(PUT));
	
	send_tuple(tuple, &mut stream);
	
	//stream.shutdown(Shutdown::Write);
}


fn main() {
    let connection = ("127.0.0.1:5000").parse().unwrap();
    //let tuple = [E::Int(10), E::Double(10.1)];
    put_tuple(&[E::Str("LOLOLOLO".to_string()), E::Int(10), E::Double(10.2), E::Str("123".to_string())], &connection);
}
