#![feature(phase)]
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;

use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};
use std::io::{File, IoError};
use std::os::getcwd;
use std::str::from_utf8;

static IP: &'static str = "127.0.0.1";
static PORT : u16 = 4414;

fn handle_request(stream: TcpStream, request_count: int) -> Result<(), IoError> {
    let mut stream = stream;

    // Print out the details of the connecting peer
    match stream.peer_name() {
        Ok(ref pn) => {println!("Received connection from: {}", pn)}
        _ => {}
    }

    // Read the request
    let mut buf = [0, ..500];
    try!(stream.read(buf));
    let request_str = from_utf8(buf).unwrap_or("");
    println!("Received request :\n{}", request_str);

    // Form the response and send it back
    let request_url = regex!(r"^GET (/.*) HTTP/1.\d").captures(request_str).unwrap().at(1);
    let file_re = regex!(r".+\.html$");

    let make_header = |code: int,details: &'static str| -> String {
        format!("HTTP/1.1 {code} {details}\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n", code=code, details=details)
    };

    let path = getcwd().join(request_url.trim_chars('/'));

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
    } else if file_re.is_match(request_url) && path.is_file() {
        response_header = make_header(200, "OK");
        String::from_str(from_utf8(File::open(&path).read_to_end().ok().unwrap().as_slice()).unwrap_or("Error"))
    } else if !path.is_file() {
        response_header = make_header(404, "Not Found");
        String::from_str("Not Found")
    } else {
        response_header = make_header(403, "Forbidden");
        String::from_str("Forbidden")
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
    let listener = TcpListener::bind(IP, PORT).ok().expect(format!("Could not bind to address {}:{}", IP, PORT).as_slice());
    let mut acceptor = listener.listen().unwrap();
    println!("Listening on [{}:{}] ...", IP, PORT);

    let mut request_count: int = 0;

    for stream in acceptor.incoming() {
        match stream {
            Ok(stream) => {
                request_count += 1;
                spawn(proc() {
                    handle_request(stream, request_count).ok().expect("Something went wrong while handling the request");
                });
            }
            _ => {}
        }
    }

    drop(acceptor);
}
