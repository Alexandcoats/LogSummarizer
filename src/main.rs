extern crate xml;

use chrono::{TimeZone, Utc};
use std::{
    collections::HashMap,
    fs::File,
    io::{prelude::*, BufReader},
};
use xml::reader::{EventReader, XmlEvent};

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    for filepath in args.iter().skip(1) {
        let path = std::path::PathBuf::from(filepath);
        let filename = path.file_name().unwrap().to_str().unwrap();
        let xml_file = File::open(filepath)?;
        let capacity = std::fs::metadata(filepath)?.len() / 5;
        let xml_file = BufReader::new(xml_file);
        let parser = EventReader::new(xml_file);
        let ts = chrono::Utc::now().timestamp();
        let mut inline = File::create(format!("{}.{}-inline.txt", ts, filename))?;
        let mut threaded = File::create(format!("{}.{}-threaded.txt", ts, filename))?;
        let mut threads = HashMap::new();
        let mut stack = Vec::new();
        let mut curr_thread = 0;
        let mut curr_line = String::new();
        for element in parser {
            match element {
                Ok(XmlEvent::StartElement { name, .. }) => stack.push(name),
                Ok(XmlEvent::EndElement { name }) => {
                    if let Some(n) = stack.pop() {
                        if name != n {
                            break;
                        } else if n.local_name == "record" {
                            let buf = threads
                                .entry(curr_thread)
                                .or_insert(String::with_capacity(capacity as usize));
                            buf.push_str(curr_line.as_str());
                            inline.write_all(curr_line.as_bytes())?;
                            curr_line = String::new();
                        }
                    }
                }
                Ok(XmlEvent::Characters(s)) => {
                    if let Some(el) = stack.last() {
                        match el.local_name.as_str() {
                            "millis" => {
                                let ts = Utc.timestamp_millis(s.parse().unwrap());
                                curr_line.push_str(
                                    format!("{} ", ts.format("%Y-%m-%d %H:%M:%S")).as_str(),
                                );
                            }
                            "thread" => {
                                curr_thread = s.parse().unwrap();
                                curr_line.push_str(format!("[{}]\t", s).as_str());
                            }
                            "message" => {
                                curr_line.push_str(format!("{}\n", s).as_str());
                            }
                            _ => (),
                        }
                    }
                }
                Err(_) => {
                    break;
                }
                _ => (),
            }
        }
        for (thread, s) in threads.iter() {
            threaded.write_all(format!("[[Thread {}]]\n", thread).as_bytes())?;
            threaded.write_all(format!("{}\n\n", s).as_bytes())?;
        }
    }
    Ok(())
}
