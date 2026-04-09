Feather Hybrid Render Engine (FHRE) Configuration for Sim Board

This configuration enables the Feather Hybrid Render Engine (FHRE) on the sim board.

Default Configuration:
- Uses the Clang version of FHRE
- Uses the Clang version of the FHRE example
- Entry point: nsh_main (NSH shell)

Switching to Rust Version:
1. Edit the defconfig file
2. Comment out the Clang version lines:
   # CONFIG_FHRE_CLANG=y
   # CONFIG_EXAMPLES_FHRE_CLANG=y
3. Uncomment the Rust version lines:
   CONFIG_FHRE_RUST=y
   CONFIG_EXAMPLES_FHRE_RUST=y
   # CONFIG_INIT_ENTRYPOINT="fhre_rust_main" (uncomment if you want Rust example as entry point)

How to use this configuration:
1. From the nuttx directory, run:
   ./tools/configure.sh sim:fhre

2. Build the project:
   make

3. Run the simulation:
   ./nuttx

4. In the NSH shell, run the FHRE demo:
   fhre_demo

This configuration will use the appropriate FHRE library and example based on the selected version (Clang or Rust).

When running fhre_demo, it will open a window and display a simple demo with a red point, green line, and blue rectangle.
