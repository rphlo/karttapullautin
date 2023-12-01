# Rusty Kartta Pullautin

Rusty-Pullauta is a application that is designed to generate highly accurate maps out of LIDAR data input files. Built using the Rust programming language, Rusty-Pullauta is an efficient fork of the Kartta-pullautin Windows software, that takes advantage of Rust's performance to deliver faster and copy conform results on Linux, Mac and Windows.

With Rusty-Pullauta, users can expect to achieve up to 10 times faster results compared to the previous software it was forked from. This is achieved through the use of Rust's ability to compile to efficient code.

Rusty-Pullauta supports a wide range of LIDAR data input file formats, including LAS, LAZ, and XYZ. The software also includes advanced algorithms for filtering, classification, and feature extraction, ensuring that users can generate highly accurate maps with ease.

In addition to its performance and accuracy, With its powerful features and fast results, Rusty-Pullauta is a must-have tool for anyone working with LIDAR data to generate orienteering maps.

### Note: The original perl script hasnt been completly ported to rust and few steps are still executed with the old perl script, however you will be able to use this program to to generate maps from start to finish.

## Usage

You can download latest binary for rust-pullauta for your platform from the latest tags.  
https://github.com/rphlo/rusty-pullauta/releases/latest

Unzip the file and copy the rusty-pullauta file to your $PATH  
e.g. `cp rusty-pullauta /usr/local/bin/`

### Dependencies
1. For the script to work you may need to install some perl script dependencies:

    `cpan install GD POSIX Config::Tiny Geo::ShapeFile`

    GD might require you to install libgd on your system

2. You'll also need the las2txt binary that you can compile with:  
    ```
    git clone https://github.com/LAStools/LAStools
    cd LAStools
    make
    cp bin/las2txt /usr/local/bin/
    ```

### Converting a LiDAR file
You can finnaly run perl script with the path to your .LAZ or .XYZ file as argument:  
`perl pullauta L3323H3.laz`

For more advanced usage read the `readme.txt` file.

## Development

Make your changes, if you modify the rust script you must run:

`cargo build --release`

Then add the new binary to your $PATH, for example:

`cp target/release/rusty-pullauta /usr/local/bin/`
