FeatherOS Wing Configuration for Sim Board

This configuration enables the Wing desktop environment demo on the sim board.

Enabled components:
- FHRE Rust library: `apps/fhre`
- Wing Rust library: `apps/wing`
- Wing Rust example: `apps/examples/wing`

How to use this configuration:
1. From the `nuttx` directory, run:
   `./tools/configure.sh sim:wing`

2. Build the project:
   `make`

3. Run the simulation:
   `./nuttx`

4. In the NSH shell, run the Wing demo:
   `wing_rust`

This will start the Wing desktop shell example built from
`apps/examples/wing`.
