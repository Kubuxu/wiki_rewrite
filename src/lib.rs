extern crate regex;
extern crate url;
extern crate walkdir;
extern crate time;

use url::percent_encoding::{
    percent_decode,
};


use std::path::Path;
use std::io::{Write, Read};
use std::borrow::Cow;
use std::hash::{Hasher, SipHasher};
use std::os::unix::fs::MetadataExt;

struct LinkReplace; //{
    //pub path: &'a Path
//}

fn map_filename(url_bytes: &[u8], levels: usize) -> String {
    assert!(levels > 0);
    assert!(levels < 8);
    let mut hasher = SipHasher::new();
    hasher.write(url_bytes);
    let hash = hasher.finish();
    let mut s = String::with_capacity(32);
    for level in 0..levels {
        let b = (hash & (0xff << (level * 8))) >> (level * 8);
        s.push_str(&format!("{:02x}/", b));
    }
    s

}

impl regex::Replacer for LinkReplace {
    fn reg_replace(&mut self, caps: &regex::Captures) -> Cow<str> {
        let orig_match = caps.at(0).unwrap();
        let orig_url: &str = caps.at(2).unwrap();
        if orig_url.starts_with("/") { return Cow::Owned(orig_match.to_owned()) }
        if orig_url.starts_with("http:") { return Cow::Owned(orig_match.to_owned()) }
        if orig_url.starts_with("https:") { return Cow::Owned(orig_match.to_owned()) }

        //println!("{:?} {:?}", self.path, orig_url);
        let url_bytes = percent_decode(orig_url.as_bytes());
        let url_str = std::str::from_utf8(&url_bytes).unwrap();
        let url_path = Path::new(url_str);
        let url_filename: &str = url_path.file_name().unwrap().to_str().unwrap();

        let orig_attr = caps.at(1).unwrap();
        // images get 1 level of partitioning
        if orig_url.starts_with("../I/m/") {
            let mapped = map_filename(url_filename.as_bytes(), 1);
            //println!("{:?} -> {:?}", orig_filename, mapped);
            return Cow::Owned(format!("{}=\"../../../I/m/{}{}\"", orig_attr, mapped, url_filename));
        } else if orig_url.starts_with("../") {
            return Cow::Owned(format!("{}=\"../../{}\"", orig_attr, orig_url));
        }


        let mapped= map_filename(&url_bytes, 2);
        //let url = percent_encode(&url_bytes, DEFAULT_ENCODE_SET);
        
        let fin = format!("{}=\"../../{}{}\"",orig_attr, mapped, orig_url);
        return Cow::Owned(fin);
    }
}

pub fn rewrite<P: AsRef<Path>>(p: P) {
    use walkdir::WalkDir;
    let start_time = time::get_time().sec;
    std::thread::sleep(std::time::Duration::new(2, 0));

    let re = regex::Regex::new(r#"(href|src)="([^ ]+)""#).unwrap();
    let mut c: f32 = 0.0;
    for entry in WalkDir::new(p) {
        let entry = entry.unwrap();
        let path = entry.path();
        //{ DEBUG
        //    let filename = path.file_name().unwrap().to_string_lossy();
        //    if  !filename.starts_with("Main_Page.html") { continue; }
        //    println!("{:?}", path);
        //}

        if let Some(ext) = path.extension() {
            if ext == "htm" || ext == "html" {
                // if this file has more than 1 links, then only modify it if its mtime is
                // older than our start time
                let md = path.metadata().unwrap();
                if md.nlink() > 1 {
                    let this_mtime = md.mtime();
                    if this_mtime >= start_time {
                       //println!("{:?} already modifying, skipping", path); 
                       continue;
                    }
                }

                // read entire file
                let html = {
                    let mut f = std::fs::File::open(&path).unwrap();
                    let mut s = String::new();
                    f.read_to_string(&mut s).unwrap();
                    s
                };

                let output = re.replace_all(&html, LinkReplace);
                if let Ok(mut out_f) = std::fs::File::create(&path) {
                    out_f.write_all(output.as_bytes()).unwrap();
                }
                c += 1.0;
                let progress: f32 = c / 5805.44;
                println!("Done {} Rewrote", progress);

            }
        }
    }

    return;

}

pub fn rename<P: AsRef<Path>>(p: P, level: usize) {
    let p: &Path = p.as_ref();
    for path in std::fs::read_dir(p).unwrap() {
        let path = path.unwrap().path();
        if !path.is_file() { continue; }

        let name = path.file_name().unwrap().to_str().unwrap();

        let mapped_dir = map_filename(&name.as_bytes(), level);

        let prefix_dir = p.join(&mapped_dir);
        if !prefix_dir.exists() {
            std::fs::create_dir_all(&prefix_dir).unwrap();
        }
        println!("Moved {:?} -> {:?}", path, prefix_dir.join(name));
        std::fs::rename(&path, prefix_dir.join(name)).unwrap();

    }

}

#[test]
fn it_works() {
}
