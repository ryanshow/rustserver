#![feature(plugin)]
#![plugin(regex_macros)]
extern crate regex;

use std::net::{TcpListener, TcpStream};
use std::fs::File;
use std::io::{Error, Read, Write};
use std::thread;
use std::env::current_dir;
use std::string::String;
use std::str::from_utf8;

static IP: &'static str = "127.0.0.1";
static PORT: u16 = 4414;

fn handle_request(mut stream: TcpStream, request_count: u8) -> Result<(), Error> {

    // Print out the details of the connecting peer
    println!("Received connection from: {}", stream.peer_addr().unwrap());

    // Read the request
    let request_buf = &mut[0u8; 500];
    try!(stream.read(request_buf));
    let request = from_utf8(request_buf).unwrap_or("");
    println!("Received request:\n{}", request);

    // Form the response and send it back
    let request_url = regex!(r"^GET (/.*) HTTP/1.\d").captures(request).unwrap().at(1).unwrap();
    let file_re = regex!(r".+\.html$");

    let make_header = |code: u16,details: &'static str| -> String {
        format!("HTTP/1.1 {code} {details}\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n", code=code, details=details)
    };

    let path = current_dir().unwrap().join(request_url.trim_matches('/'));

    let response_header;

    let response_body = if request_url == "/" {
        response_header = make_header(200, "OK");
        format!(
            "<doctype !html><html><head><title>Hello, Rust!</title>
            <style>body {{ background-color: #111; color: #FFEEAA }}
            h1 {{ font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm red}}
            h2 {{ font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm green}}
            </style></head>
            <body>
            <h1>Greetings, Krusty!</h1>
            <h2>Visitor Count: {visitor_count}</h2>
            </body></html>\r\n"
        , visitor_count=request_count)
    } else if file_re.is_match(request_url) {
        let mut file_string = String::new();
        match File::open(&path).as_mut() {
            Ok(file) => {
                response_header = make_header(200, "OK");
                let _ = file.read_to_string(&mut file_string);
                file_string
            }
            _ => {
                response_header = make_header(404, "Not Found");
                "File Not Found".to_string()
            }
        }
    } else {
        response_header = make_header(403, "Forbidden");
        "Forbidden".to_string()
    };

    let response = format!("{header}{body}",
        header=response_header,
        body=response_body
    );

    try!(stream.write(response.as_bytes()));

    println!("Connection terminates.");

    return Ok(());
}

fn main() {
    let listener = TcpListener::bind((IP, PORT)).unwrap();
    println!("Listening on [{}:{}] ...", IP, PORT);

    let mut request_count: u8 = 0;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                request_count += 1;
                thread::spawn(move|| {
                    let _ = handle_request(stream, request_count);
                });
            }
            _ => {/* connection failed */}
        }
    }
    drop(listener);
}
