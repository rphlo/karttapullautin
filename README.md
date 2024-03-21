# Rusty Kartta Pullautin
### ***Rusty Map Generator***

Rusty-Pullauta is an application that is designed to generate highly accurate maps out of LiDAR data input files. Built using the Rust programming language, Rusty-Pullauta is an efficient rewrite of the Kartta-pullautin Windows software originally written in perl. It takes advantage of Rust's performance to deliver faster and copy conform results on Linux, Mac and Windows.

With Rusty-Pullauta, users can expect to achieve up to 10 times faster results compared to the perl predecessor. This is achieved through the use of Rust's ability to compile to efficient code.

Rusty-Pullauta supports a wide range of LiDAR data input file formats, namely LAS, LAZ, and XYZ. The software uses advanced algorithms for filtering, classification, and feature extraction, ensuring that users can generate highly accurate maps.

Due to its performance and accuracy, with its powerful features and fast results, Rusty-Pullauta is the must-have tool to automatically generate orienteering maps from LiDAR data.

***Note: The original perl script has not been yet completly ported to rust, the shape files drawing step is still executed with a portion of perl script***

## Usage
You can download and extract the latest binary of rust-pullauta for your platform from the latest releases.

See: https://github.com/rphlo/rusty-pullauta/releases/latest

### Dependencies
1. You'll need the las2txt binary that you can compile with:  
    ```
    git clone https://github.com/LAStools/LAStools
    cd LAStools
    make
    cp bin/las2txt /usr/local/bin/
    ```

2. If you want to use the shape file drawing step on linux or mac, for the script to work, you will need to install some perl script dependencies:

    `cpan install GD POSIX Config::Tiny Geo::ShapeFile`

    GD might require you to install libgd on your system

### Converting a LiDAR file
You can run the rusty-pullauta executable with the path to your .LAZ, .LAS or .XYZ file as argument:  
`rusty-pullauta L3323H3.laz`

For more advanced usage read the `readme.txt` file.

## Development
Make your changes, if you modify the rust script you must run:

`cargo build --release`

Then copy the executable to your current directory:

`cp target/release/rusty-pullauta .`
