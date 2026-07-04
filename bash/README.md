## Bash

You can run either `build.sh` or `run.sh` from the root of the project. Running them from inside this folder will not work.

Example: `./bash/build.sh`

### build.sh

This will build and package both `nimbus-client` and `nimbus-server` into a .ZIP file at the root of the project.

Use the `--build-NT` argument to create a Windows .ZIP instead. You will need the `x86_64-pc-windows-gnu` toolchain.

### run.sh

Launch both a Nimbus server and a client instance for development. The server will run in the background and terminate when the client instance does.