use quick_xml::Writer;
use quick_xml::events::{ Event, BytesStart };
use quick_xml::reader::Reader;
use quick_xml::name::QName;
use quick_xml::events::attributes::Attribute;
use std::io::Cursor;
use std::{str, fs};

use regex::Regex;


/// Taken from [StackOverflow](https://stackoverflow.com/a/65976629/20675205)
fn cut_first_and_last_char(s: &str) -> &str {
    let mut chars = s.chars();
    chars.next();
    chars.next_back();
    chars.as_str()
}

fn main() {
    let path = "./my-workplace.svg";

    let mut reader = Reader::from_file(path).unwrap(); // xml:&str

    let mut buf = Vec::new();

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            // exits the loop when reaching end of file
            Ok(Event::Eof) => break,

            Ok(Event::Start(e))/* if...? */ => {
                // check if both `id` and `inkscape:label` are set
                // taken from https://stackoverflow.com/a/70029816/20675205
                let has_id_and_label = e.attributes()
                    // assumption: attr-names are unique!
                    .any(|attr| attr.unwrap().key == QName(b"inkscape:label"))
                ;
                if has_id_and_label {
                    // replace `id`-value with value of `inkscape:label`
                    // crates a new element ... alternatively we could reuse `e` by calling
                    // `e.into_owned()`
                    let mut elem = BytesStart::new(str::from_utf8(e.name().into_inner()).unwrap());

                    let inkscape_label = e.attributes()
                        .filter(|a| a.clone().unwrap().key==QName(b"inkscape:label"))
                        .map(|a| a.clone().unwrap())
                        .collect::<Vec<Attribute>>().get(0).unwrap().value.clone().into_owned();
                    let inkscape_label_val = String::from_utf8(inkscape_label).unwrap();
                    // println!("RandomHash2423432422 {inkscape_label_val}");
                    // collect existing attributes
                    elem.extend_attributes(e.attributes()
                        .filter(|a| a.clone().unwrap().key != QName(b"id"))
                        .map(|a| a.unwrap()))
                    ;

                    // copy existing attributes, adds a new my-key="some value" attribute
                    elem.push_attribute(("id", inkscape_label_val.as_str()));
                    // writes the event to the writer
                    assert!(writer.write_event(Event::Start(elem)).is_ok());
                }else {
                    let elem = e.into_owned();
                    assert!(writer.write_event(Event::Start(elem)).is_ok());
                }

            }

            // There are several other `Event`s we do not consider here
            Ok(other) => {
                assert!(writer.write_event(other).is_ok());
            },
        }
        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }
    let result = writer.into_inner().into_inner();
    let res_str = String::from_utf8(result).unwrap();

    // 1. Replace `\n` with `\r`
    let res_str = str::replace(res_str.as_str(), r#"\n"#, r#"\r"#);
    // 2. Replace `\"` with `"`
    let res_str = str::replace(res_str.as_str(), r#"\""#, r#"""#);
    println!("{res_str:?}");
    fs::write("./test2.svg", res_str).unwrap();
}
