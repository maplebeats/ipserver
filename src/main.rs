use futures::executor;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::env;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

fn main() {
    simple_logger::init_by_env();
    let args: Vec<String> = env::args().collect();
    let port: String;
    if args.len() < 2 {
        port = "8080".to_string();
    } else {
        port = args[1].to_string();
    }
    info!("server running at {}", port);
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(addr).unwrap();
    for stream in listener.incoming() {
        let handle = async {
            match stream {
                Ok(stream) => handle_connection(stream),
                Err(err) => error!("stream err {}", err),
            };
        };
        executor::block_on(handle);
    }
}
fn handle_connection(mut stream: TcpStream) {
    let mut buff = [0; 512];
    let n = stream.read(&mut buff[..]).unwrap_or_default();
    debug!("read header length:{}", n);
    let request = String::from_utf8_lossy(&buff[..]);
    let mut reqs = request.lines();
    let mut headers = HashMap::new();
    let mut index = 0;
    let mut method = "GET".to_string();
    loop {
        match reqs.next() {
            Some(req) => {
                let mut v = req.split_whitespace();
                let key = v.next().unwrap_or_default();
                let value = v.next().unwrap_or_default();
                if index == 0 {
                    method = key.to_string();
                }
                debug!("{} {}", key, value);
                if key.trim().len() != 0 && value.trim().len() != 0 {
                    debug!("insert-> '{}' '{}'", key, value);
                    headers.insert(key, value);
                }
            }
            None => break,
        }
        index = index + 1;
    }
    handle_path(method, headers, stream);
}
fn handle_path(method: String, headers: HashMap<&str, &str>, mut stream: TcpStream) {
    let peer_addr = stream.peer_addr().unwrap();
    let response: String;
    let ua = match headers.get("User-Agent:") {
        Some(ua) => ua,
        None => {
            warn!("{} not found ua", peer_addr);
            ""
        }
    };
    if is_cmd(ua.to_string()) {
        response = build_http(peer_addr.to_string());
    } else {
        response = build_http(build_body(peer_addr.to_string()));
    }
    info!("{} from {} ua {}", method, peer_addr, ua);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn favicon() -> String {
    let base="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAABmJLR0QA/wD/AP+gvaeTAAAD2klEQVR4nO2aTWgUSRiGn5qejFkyHvxbNWYXUdSoYffgydV4cHYPi+BJT/4gCEERLyqoJ0UvgrgLK+zqKZBrQMRLRInsEl3ZUw4Je/NkImgSPSma2P15mBnS6cxPV1d1Twf7hT5UVVd9b79dX1W/3Q0ZMmTIkOHrhdLtsPOqFN6/4brkOIawPg5SAbwTeOAVODfxu3pre3BtATadkRsIF20TCYFx5VF6cUe9sTmotgBbTskk0Onl2PPiT/WvTTJ14omvOO5hV4ScbgdH6HQEkrj4SrzqMe4IPW3C8OZT8q2t8aMIgCPNz7OFarxvZtnvCGOO0NPu8c/2PrGy/mgLkPfKR1KoxhvtV1OFOUp5j7G8R3fB48kuCyIsmRkAZRFyLiUHxhyhWz6bi6AvgFc+kkIw3mi/mhKXkuOVRcjNmYmwpGZAFaP9aqodSpU1obttlie7jkYTYSkI8MoR2Htc9vrrR8rp8HNld+hud3i0+6Ss1B0/r00owelfiTcAXAJG9h2rofw8nx7nE78BJ3TG1xYgn+DdB1j7kSsz7YBwHOhscvpB3fH1Z0DCAgwOqlngcuWoi1+OiAArdMdPfQqERVReqZ8BYRGVVywCnH8uZxH+AEB4emuP6q22XXgmB0RxF9hQqZpA0XdrtxoK027CqxZiSYGOOVDzPnPB9tXhcYf5iwPoUnAX+D5MuwmvWohlF7i2T90Gbt/8WxadXXTpAnAKrPr4AdWWZxr4Lmy7Ca+a/XQ76ChddOvXnf6p/Hbnr2HRarfBy49YF8FGAkQt2+DlR6wCLK9xV4J1umUbvPxoWQpELdvg5UfLUuC/e7IKQOpccL12G7z8aIUAE0AXimkAyue8DNtug5cfsb4SK7rMFF1mFtR9pq/oMlF0ywItd3nZIfSFbbfBa0E/3Q46Sm88pFYvqjushmiwrzdrt8HLj8wLaAfK3GC480xNTyNDZcIriNhmgKnpaWSoTHgFEdsrMVPT08hQmfBa1E+3Q9ipZsv0pM8LhH8GMCo3q4/KK4jYZoAt05M+M6SZAlHLzeqj8goi9hQwNT2pSwGNXcCK6QkrQNRdILZvg7ZMTy1DZcIriNhSwJbpqWWoTHgFkZkh7UCZGYoWKG4kNgOS/jweFqn8MJIkshRI+jng3o/S1KMnhcEfpLcVzwEj93tSMh0M0tIkBV4Da6OHtorIXCLvAr/+r9ZFCRgXhnaEe3MURJQ/RV85Hjzemp414OEO6a38UTqp21c/BWAA4RKKkeFtKVkDKo5RhAHdrtoCrHO4Mj0HEu6/vaQwiTCwZhlXW00kQ4YMGTIsJXwBYqXyAwsO2h0AAAAASUVORK5CYII=";
    let link = format!("<link rel=\"icon\" href=\"{}\">", base);
    link.to_string()
}
fn build_http(body: String) -> String {
    format!("HTTP/1.1 200 ok\r\n\r\n{}", body)
}
fn build_body(context: String) -> String {
    let body = format!(
        "<html>
    <head>
        {}
    </head>
    <body>
        <h1>{}</h1>
    </body>
</html>
    ",
        favicon(),
        context
    );
    body.to_string()
}

fn is_cmd(ua: String) -> bool {
    debug!("check ua {}", ua);
    let commands = ["curl", "wget", "apache"];
    let mut found: bool;
    found = false;
    for command in &commands {
        match ua.to_lowercase().find(command) {
            Some(_i) => found = true,
            None => continue,
        }
    }
    found
}
