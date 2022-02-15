//! Custom Panic Messages According to the error.
//!
//! Handles panics by calling a custom function using
//! [`std::panic::set_hook`](https://doc.rust-lang.org/std/panic/fn.set_hook.html)
//! and a Yaml File to generate the custom structs.
//!
//! This allows for seperate error messages for seperate error and also allows the user to run some simple fixes (if possible).
//!
//! ### Output Example
//!
//! Example of an API error's panic output
//!
//! ```txt
//! The Program Crashed
//!
//! Error: There was an error during the API request
//! It seems like an error that can be fixed by you!
//! Please follow the following instructions to try and fix the Error
//!
//!     1: Try to check your Internet Connection.
//!
//! 	2: Check if your API request quota has been exhausted.
//! 		1.  Instructions on how
//! 		2.  to check
//! 		3.  API quota
//!
//! If the error still persists
//! Contact the Developer at xyz@wkl.com
//! ```
//! ### Code Example
//! To replicate the above output you need to first create a yaml file as follows.
//! ```txt
//! API:
//!   message: There was an error during the API request
//!   fix instructions:
//!       - Try to check your Internet Connection.
//!       - Check if your API request quota has been exhausted.
//!       - - Instructions on how
//!         - to check
//!         - API quota
//! ```
//! then you need to create the [build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html) make sure userpanic is present in both dependencies and build dependencies in cargo.toml file
//! ```toml
//! [dependencies]
//! userpanic = "0.1.0"
//!
//! [build-dependencies]
//! userpanic = "0.1.0"
//! ```
//! and make build.rs file as follows
//! ```
//! fn main() {
//!    println!("cargo:rerun-if-changed=errors.yaml");
//!    println!("cargo:rerun-if-changed=build.rs");
//!    userpanic::panic_setup!("errors.yaml"); // Enter the yaml file path here
//! }
//! ```
//! This will create `panic_strucs.rs` file in src directory
//! This file can be then imported and used with panic_any to display the custom panics
//! ```
//! mod panic_structs;
//!
//! use std::panic::panic_any;
//! use crate::panic_structs::API;
//!
//! fn main(){
//!     // This sets the custom hook for panic messages
//!     userpanic::set_hooks(Some("If the error still persists\nContact the developer at xyz@wkl.com"));
//!     // If None is passed then No developer info/message is shown.
//!
//!     panic_any(API);
//! }
//! ```

use log::{debug, info};
use std::fmt;
use std::io::Write;
use std::panic;
use std::panic::PanicInfo;
use yaml_rust::{Yaml, YamlLoader};

type StrList = [&'static [&'static str]];
type Panicfn = Box<dyn Fn(&PanicInfo) + Sync + Send>;

#[derive(Debug, Clone)]
/// This Struct is auto generated from the yaml file
pub struct UserPanic {
    /// It describes the error
    ///
    /// If left empty then the program panics silently without giving any output
    pub error_msg: &'static str,
    /// It contains the instructions to fix the error
    pub fix_instructions: Option<&'static StrList>,
}
impl fmt::Display for UserPanic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.error_msg == "" {
            return write!(f, "");
        }
        // Need something better than "The Program Crashed" :(
        let mut s = String::from("The Program Crashed\n\n");
        if self.fix_instructions.is_none() {
            s += &format!("Error: {}", self.error_msg);
            s += "\nIt seems like an error that can't be fixed by you!\nPlease submit a Bug report to Developer\n";
        } else {
            s += &format!("Error: {}", self.error_msg);
            s += "\nIt seems like an error that can be fixed by you!\nPlease follow the following instructions to try and fix the Error\n";
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
/// This function is used to set custom panic function
/// Use this to use the custom hooks and set up the developer message
pub fn set_hooks(developer: Option<&'static str>) {
    let org: Panicfn = panic::take_hook();
    if let Some(dev) = developer {
        // Used if The developer provides custom info
        panic::set_hook(Box::new(move |pan_inf| {
            panic_func(pan_inf, &org);
            eprintln!("{}", dev);
        }))
    } else {
        // Used if Developer doesn't want info to be shown.
        panic::set_hook(Box::new(move |pan_inf| {
            panic_func(pan_inf, &org);
        }));
    }
}
// The panic function
fn panic_func(panic_info: &PanicInfo, original: &Panicfn) {
    match panic_info.payload().downcast_ref::<UserPanic>() {
        Some(err) => {
            if err.error_msg != "" {
                eprintln!("{}", err);
            }
        }
        // Default to original panic routine if downcast_ref fails
        None => original(panic_info),
    }
}
// Returns the auto generated rust code
fn read_from_yml(yaml: String) -> String {
    debug!("Started Reading the yaml string");
    let mut file = "use userpanic::UserPanic;\n".to_string();
    let yaml = YamlLoader::load_from_str(&yaml).unwrap();
    let structs = &yaml[0];
    if let Yaml::Hash(hash) = structs {
        info!("Found Hash");
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
// Helper function for read_from_yml
// Idk why I named it this it doesn't make sense
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
/// Macro to be used in build script
/// Only yaml file path or both yaml and output rust file can be provided
macro_rules! panic_setup {
    ($file_path:expr) => {
        userpanic::panic_setup_function($file_path, "src/panic_structs.rs");
    };
    ($file_path:expr,$file_out:expr) => {
        userpanic::panic_setup_function($file_path, $file_out);
    };
}
/// Not intended to be used directly and to be called by panic_setup! macro
/// The main build script function
pub fn panic_setup_function(path_from: &str, path_to: &str) {
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
}
