# user-panic

Custom Panic Messages According to the error.

Handles panics by calling a custom function using
[`std::panic::set_hook`](https://doc.rust-lang.org/std/panic/fn.set_hook.html)
and a Yaml File to generate the custom structs.

This allows for seperate error messages for seperate error and also allows the user to run some simple fixes (if possible).

#### Output Example

Example of an API error's panic output

```txt
The Program Crashed

Error: There was an error during the API request
It seems like an error that can be fixed by you!
Please follow the following instructions to try and fix the Error

    1: Try to check your Internet Connection.

	2: Check if your API request quota has been exhausted.
		1.  Instructions on how
		2.  to check
		3.  API quota

If the error still persists
Contact the Developer at xyz@wkl.com
```
#### Code Example
To replicate the above output you need to first create a yaml file as follows.
```txt
API:
  message: There was an error during the API request
  fix instructions:
      - Try to check your Internet Connection.
      - Check if your API request quota has been exhausted.
      - - Instructions on how
        - to check
        - API quota
```
then you need to create the [build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html) make sure userpanic is present in both dependencies and build dependencies in cargo.toml file
```toml
[dependencies]
user-panic = "0.1.0"

[build-dependencies]
user-panic = "0.1.0"
```
and make build.rs file as follows
```rust
fn main() {
   println!("cargo:rerun-if-changed=errors.yaml");
   println!("cargo:rerun-if-changed=build.rs");
   userpanic::panic_setup!("errors.yaml"); // Enter the yaml file path here
}
```
This will create `panic_strucs.rs` file in src directory
This file can be then imported and used with panic_any to display the custom panics
```rust
mod panic_structs;

use std::panic::panic_any;
use crate::panic_structs::API;

fn main(){
    // This sets the custom hook for panic messages
    userpanic::set_hooks(Some("If the error still persists\nContact the developer at xyz@wkl.com"));
    // If None is passed then No developer info/message is shown.

    panic_any(API);
}
```
