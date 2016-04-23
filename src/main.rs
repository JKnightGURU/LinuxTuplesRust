use std::net::*;
use std::io::prelude::*;
use std::mem::*;
use std::vec::Vec;

extern crate linux_tuples_client;
use linux_tuples_client::*;

fn main() {
    let conn: SocketAddr = ("127.0.0.1:5000").parse().unwrap();
    let tuple = vec![E::I(10), E::D(10.1), E::S("123".to_string())];
    let tuple2 = vec![E::S("12789".to_string()), E::I(10), E::D(5.), E::S("1234".to_string())];
    let tuple3 = vec![E::S("12abv".to_string()), E::I(11), E::D(5.), E::S("123".to_string())];

    let serv = LinuxTuplesConnection { connection: conn };

    serv.put_tuple(&tuple);
    serv.put_tuple(&tuple2);
    serv.put_tuple(&tuple3);

    let templates: Vec<Vec<E>> = vec![];
    let tuples_dump = serv.read_all_tuples(&templates).unwrap();

    for tuple in tuples_dump {
        for item in tuple {
            item.println();
        }
        println!("");
    }

    println!("Count: {}", serv.number_of_tuples(&templates).unwrap());

    serv.replace_tuple(&tuple2, &tuple3);

    let tuples_dump = serv.read_all_tuples(&templates).unwrap();

    for tuple in tuples_dump {
        for item in tuple {
            item.println();
        }
        println!("");
    }

    println!("Count: {}", serv.number_of_tuples(&templates).unwrap());
    
    println!("{}", serv.server_log().unwrap());

}
