extern crate multiqueue;

extern crate hyper;
extern crate hyper_native_tls;
use hyper::Client;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use std::io::Read;
use std::thread;

#[macro_use]
extern crate html5ever;

use std::default::Default;
use std::string::String;

use html5ever::parse_document;
use html5ever::rcdom::{Handle, NodeData, RcDom};
use html5ever::tendril::TendrilSink;

extern crate string_cache;

fn http_get(url: &str) -> String {
    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);

    let mut resp = client.get(url).send().unwrap();
    let mut body = vec![];
    resp.read_to_end(&mut body).unwrap();

    return String::from_utf8_lossy(&body).to_string();
}

fn parse_links(body: String) -> Vec<String> {
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut body.as_bytes())
        .unwrap();

    let mut links: Vec<String> = Vec::new();
    let mut queue: Vec<Handle> = Vec::new();

    let link = string_cache::Atom::from("a");
    let href = string_cache::Atom::from("href");

    queue.push(dom.document);

    while queue.len() != 0 {
        let node = queue.remove(0);
        match node.data {
            NodeData::Element {
                ref name,
                ref attrs,
                ..
            } if name.local == link =>
            {
                assert!(name.ns == ns!(html));
                for attr in attrs.borrow().iter() {
                    assert!(attr.name.ns == ns!());
                    if attr.name.local == href {
                        links.push(String::from(attr.value.clone()));
                        break;
                    }
                }
            }
            _ => {}
        }
        for child in node.children.borrow().iter() {
            queue.push(child.clone());
        }
    }
    return links;
}

fn main() {
    let (send, recv) = multiqueue::mpmc_queue(4);
    let mut handles = vec![];

    for _ in 0..2 {
        let stream_consumer = recv.clone();
        handles.push(thread::spawn(move || {
            for val in stream_consumer {
                let body = http_get(val);
                let links = parse_links(body);
                println!("{:?}", links);
            }
        }));
    }
    recv.unsubscribe();

    loop {
        if send.try_send("https://au.jora.com").is_ok() {
            break;
        }
    }
    drop(send);

    for t in handles {
        if !t.join().is_ok() {
            panic!()
        }
    }
}
