use log::{debug, info};
use std::fmt;
use std::io::Write;
use std::panic;
use std::panic::PanicInfo;
use yaml_rust::{Yaml, YamlLoader};

type StrList = [&'static [&'static str]];

#[derive(Debug, Clone)]
pub struct UserPanic {
    pub error_msg: &'static str,
    pub fix_instructions: Option<&'static StrList>,
}
impl fmt::Display for UserPanic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.error_msg == "" {
            return write!(f, "");
        }
        let mut s = String::from("Whoops!\nUnrecoverable error occerred\n\n");
        if self.fix_instructions.is_none() {
            s += &format!("Error: {}", self.error_msg);
            s += "\nIt seems like an error that can't be fixed by you!\nPlease follow the following instructions to submit a Bug report to Developer\n";
        } else {
            s += &format!("Error: {}", self.error_msg);
            s += "\nIt seems like an error that can be fixed by you!\nPlease follow the following instructions to try and fix the bug\n";
            let insts = self.fix_instructions.as_ref().unwrap();
            let mut i = 1;
            for inst in *insts {
                s += &format!("\n\t{}: {}\n", i, inst[0]);
                let inst = &inst[1..];
                if inst.len() > 1 {
                    let mut j = 1;
                    for ii in inst {
                        s += &format!("\t\t{}.  {}\n", j, ii);
                        j += 1;
                    }
                }
                i += 1;
            }
        }
        write!(f, "{}", s)
    }
}

pub fn set_hooks(developer: Option<&'static str>) {
    let _org = panic::take_hook();
    if let Some(dev) = developer {
        // Used if The developer provides custom info
        //
        panic::set_hook(Box::new(move |pan_inf| {
            panic_func(pan_inf);
            eprintln!("{}", dev);
        }))
    } else {
        // Used if Developer doesn't want info to be shown.
        panic::set_hook(Box::new(panic_func));
    }
}

fn panic_func(panic_info: &PanicInfo) {
    match panic_info.payload().downcast_ref::<UserPanic>() {
        Some(err) => {
            if err.error_msg != "" {
                eprintln!("{}", err);
            }
        }
        None => match panic_info.payload().downcast_ref::<&str>() {
            Some(err) => eprintln!("{}", err),
            None => match panic_info.payload().downcast_ref::<String>() {
                Some(err) => eprintln!("{}", err),
                None => eprintln!("Unrecognized ERROR format"),
            },
        },
    }
}

fn read_from_yml(yaml: String) -> String {
    debug!("Started Reading the yaml string");
    let mut file = "use userpanic::UserPanic;\n".to_string();
    let yaml = YamlLoader::load_from_str(&yaml).unwrap();
    let structs = &yaml[0];
    if let Yaml::Hash(hash) = structs {
        let l = hash.keys().clone();

        info!("Found Hash with keys");
        for ll in l {
            debug!("{:?}", ll);
        }
        // for test case keys -> foo bar
        for (key, val) in hash {
            let st_name = key.as_str().unwrap();
            debug!("parsing key {}", st_name);
            file += &format!(
                "pub const {}:UserPanic = UserPanic {{{}}};",
                st_name,
                get_err_msg(val)
            );
        }
    }
    file
}
fn get_err_msg(hash: &Yaml) -> String {
    let print_arr = |arr: &Vec<Yaml>| -> String {
        let mut s = String::new();
        let _ = arr
            .iter()
            .map(|a| {
                s += &format!(",\"{}\"", a.as_str().unwrap());
            })
            .collect::<Vec<_>>();
        s
    };
    let mut s = String::new();
    debug!("found hash {:#?}", hash);
    let err_ms = hash["message"].as_str().unwrap();
    debug!("Collecting  err message: {}", err_ms);
    debug!("{:?}", &hash["fix instructions"]);
    if let Yaml::Array(arr) = &hash["fix instructions"] {
        debug!("Found fix instructions");
        s += &format!("error_msg:\"{}\",fix_instructions:Some(&[", err_ms);
        let items = arr.len();
        debug!("Number of instuctions {}", items);
        let mut i = 0;
        while i < items {
            if i + 1 < items {
                match &arr[i + 1] {
                    Yaml::String(_) => {
                        s += &format!("&[\"{}\"],", arr[i].as_str().unwrap());
                        i += 1;
                    }
                    Yaml::Array(ar) => {
                        s += &format!("&[\"{}\"{}],", arr[i].as_str().unwrap(), print_arr(ar));
                        i += 2;
                    }
                    _ => {}
                }
            } else {
                match &arr[i] {
                    Yaml::String(ss) => {
                        s += &format!("&[\"{}\"],", ss);
                        i += 1;
                    }
                    Yaml::Array(ar) => {
                        s += &format!("&[\"{}\"{}],", arr[i].as_str().unwrap(), print_arr(ar));
                        i += 2;
                    }
                    _ => {}
                }
            }
        }
        s += "]),";
    } else {
        s += &format!("error_msg:\"{}\",fix_instructions: None,", err_ms);
    }
    s
}

#[macro_export]
macro_rules! panic_setup {
    ($file_path:expr) => {
        userpanic::panic_setup_default($file_path);
    };
    ($file_path:expr,$file_out:expr) => {
        userpanic::panic_setup_with_path($file_path, $file_out);
    };
}

pub fn panic_setup_default(path: &str) {
    let file_str = std::fs::read_to_string(path).expect("Failed to read yaml file");
    let s = read_from_yml(file_str);
    let mut fp =
        std::fs::File::create("src/panic_structs.rs").expect("failed to create output file");
    write!(&mut fp, "{}", s).expect("failed to write to file");
}
pub fn panic_setup_with_path(path_from: &str, path_to: &str) {
    let file_str = std::fs::read_to_string(path_from).expect("Failed to read yaml file");
    let s = read_from_yml(file_str);
    let mut fp = std::fs::File::create(path_to).expect("failed to create output file");
    write!(&mut fp, "{}", s).expect("failed to write to file");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[should_panic]
    fn it_works() {
        const ERROR: UserPanic = UserPanic {
            error_msg: "This is an error",
            fix_instructions: Some(&[
                &["Only one"],
                &["one", "two", "tem"],
                &["bem", "lem", "jem"],
            ]),
        };

        set_hooks(None);
        std::panic::panic_any(ERROR);
    }

    #[test]
    fn print_s() {
        //        env_logger::init();
        let s = "
foo:
    message: this is the main error
    fix instructions:
        - first
        - - in first
          - in first second
        - second
        - - second first
          - second second
        - third
bar:
    message: This is un fixable error
";
        let s = read_from_yml(s.to_string());
        assert_eq!("use userpanic::UserPanic;\npub const foo:UserPanic = UserPanic {error_msg:\"this is the main error\",fix_instructions:Some(&[&[\"first\",\"in first\",\"in first second\"],&[\"second\",\"second first\",\"second second\"],&[\"third\"],]),};pub const bar:UserPanic = UserPanic {error_msg:\"This is un fixable error\",fix_instructions: None,};", s);
    }

    #[test]
    fn from_file() {
        let file_str = std::fs::read_to_string("s.yaml").unwrap();
        let s = read_from_yml(file_str);
        assert_eq!("use userpanic::UserPanic;\npub const ERROR:UserPanic = UserPanic {error_msg:\"This is an error\",fix_instructions:Some(&[&[\"Main Cause\",\"Main Cause 1\",\"Main Cause 2\",\"Main Cause 3\"],&[\"Secondary Cause\"],&[\"Tert Cause\",\"Tert Cause 1\",\"Tert Cause 2\"],]),};pub const SILENT:UserPanic = UserPanic {error_msg:\"\",fix_instructions: None,};", s);
    }
}
