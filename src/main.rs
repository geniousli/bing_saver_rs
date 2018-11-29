extern crate hyper;
extern crate serde;
extern crate victoria_dom;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::time::{Duration, SystemTime};

use hyper::client::Client;
use hyper::rt::{self, Future, Stream};
use hyper::Body;
use hyper::Chunk;
use hyper::Request;
use hyper::Uri;
use serde_json::{Error, Value};
use std::fs::File;
use std::io::prelude::*;
use std::str;
use victoria_dom::DOM;

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    idx: i32,
    format: String,
    n: i32,
    nc: u64,
    pid: i32,
}
static HOSTURL: &'static str = "http://cn.bing.com/";

// impl hyper::body::Payload for Data {
//     type Data = Chunk;
//     type Error = Error;

//     fn poll_data(&mut self) -> Poll<Option<Self::Data>, Self::Error> {
//         let json = serde_json::to_string(&self).unwrap();
//         json.to_bytes();
//     }
// }
// let data = Data {
//     idx: 0,
//     format: String::from("js"),
//     n: 1,
//     nc: timestamp,
//     pid: 1,
// };

// request.body_mut().push_str(json.as_str());
// let json = serde_json::to_string(&data).unwrap();
// let request = Request::builder()
//     .method("GET")
//     .uri("http://cn.bing.com/HPImageArchive.aspx")
//     .body(String::from(""))
//     .unwrap();
// println!("{:?}", request);

fn main() {
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let timestamp = duration.as_secs();
    for i in 0..7 {
        let url = format!(
            "{}HPImageArchive.aspx?format=js&idx={}&n=1&nc={}",
            HOSTURL, i, timestamp
        );
        download_img(url);
    }
}

fn download_img(url: String) {
    let mut request = Request::new(Body::empty());
    *request.uri_mut() = url.parse::<Uri>().unwrap();
    let client = Client::builder().keep_alive(false).build_http();

    let fut = client
        .request(request)
        .and_then(|res| res.into_body().concat2())
        .and_then(|body| {
            let data = ::std::str::from_utf8(&body).expect("httpbin sends utf-8 JSON");
            println!("body: {}", data);
            let hash: Value = serde_json::from_str(data).unwrap();
            let url = hash
                .get("images")
                .unwrap()
                .get(0)
                .unwrap()
                .get("url")
                .unwrap()
                .as_str()
                .unwrap();

            let pic_url = format!("{}{}", HOSTURL, url);
            let pic_client = Client::builder().keep_alive(false).build_http();
            let mut pic_request = Request::new(Body::empty());
            *pic_request.uri_mut() = pic_url.parse::<Uri>().unwrap();

            let url_split = url.split("/").collect::<Vec<&str>>();
            let file_name = Box::new(url_split.last().unwrap().to_string());
            let fu = pic_client
                .request(pic_request)
                .and_then(|res| res.into_body().concat2())
                .and_then(move |body| {
                    let mut file = File::create(file_name.to_string()).unwrap();
                    file.write_all(&body);
                    println!("------------done ");
                    Ok(())
                })
                .map_err(|err| {
                    println!("error: {}", err);
                });

            rt::spawn(fu);
            Ok(())
        })
        .map_err(|err| {
            println!("error: {}", err);
        });
    rt::run(fut);
}
