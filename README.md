# Rusty Kartta Pullautin
### ***Rusty Map Generator***

Rusty-Pullauta is a application that is designed to generate highly accurate maps out of LiDAR data input files. Built using the Rust programming language, Rusty-Pullauta is an efficient transcription of the Kartta-pullautin Windows software that takes advantage of Rust's performance to deliver faster and copy conform results to original software on Linux, Mac and Windows.

With Rusty-Pullauta, users can expect to achieve up to 10 times faster results compared to the original perl software.  
This is achieved through the use of Rust's ability to compile to efficient code.

Rusty-Pullauta supports a wide range of LiDAR data input file formats, namely LAS, LAZ, and XYZ files. The software includes advanced algorithms for filtering, classification, and feature extraction, ensuring that users can generate highly accurate maps with ease.

Due to its performance and accuracy, with its powerful features and fast results, Rusty-Pullauta is a must-have tool for anyone willing to generate orienteering maps from LiDAR data to generate .

## Usage
You can download and extract the latest binary of rusty-pullauta for your platform from the latest releases.
See: https://github.com/rphlo/rusty-pullauta/releases/latest

### Converting a LiDAR file
You can run the rusty-pullauta executable with the path to your .LAZ, .LAS or .XYZ file as argument:  
`rusty-pullauta L3323H3.laz`

For more advanced usage such as batch mode read the `original-perl-readme.txt` file.

### Note:
Some commands from the original kartta pullata that are not necessary for the map generation are not supported by rusty-pullauta:  
They are:
  - cliffgeneralize
  - ground
  - ground2
  - groundfix
  - makecliffsold
  - makeheight
  - makevege
  - vege
  - profile
  - xyzfixer

## Development

Make your changes, then youd run:

`cargo build --release`

The new binary will be accessible in the `target/release/` directory
