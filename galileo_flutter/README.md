## Galileo flutter integration

### Architecture
- core logic in `./rust`
- interop functions and types under `./rust/api`
- flutter abstractions in `./lib`


### How to build 
1. rust obviously 
1. install flutter 3.29 (should prob work for any recent version tho) from https://docs.flutter.dev/install/archive
1. if you change anything related to the rust interop with flutter you'd need to install `flutter_rust_bridge` and run `flutter_rust_bridge_codegen generate`

### Run the example app
1. `cd ./example && flutter run` you'd need to use the desktop variant when prompted (not the web one).
