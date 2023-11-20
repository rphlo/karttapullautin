# Rusty Kartta Pullautin

Rusty-Pullauta is a application that is designed to generate highly accurate maps out of LIDAR data input files. Built using the Rust programming language, Rusty-Pullauta is an efficient fork of the Kartta-pullautin Windows software, that takes advantage of Rust's performance to deliver faster and copy conform results on Linux, Mac and Windows.

With Rusty-Pullauta, users can expect to achieve up to 10 times faster results compared to the previous software. This is achieved through the use of Rust's ability to compile to efficient, low-level code.

Rusty-Pullauta supports a wide range of LIDAR data input file formats, including LAS, LAZ, and XYZ. The software also includes advanced algorithms for filtering, classification, and feature extraction, ensuring that users can generate highly accurate maps with ease.

In addition to its performance and accuracy, With its powerful features and fast results, Rusty-Pullauta is a must-have tool for anyone working with LIDAR data to generate orienteering maps.

### Warning: this app is in active development phase, currently few steps are still executed with the old perl script, however the full process can already be used to generate maps.

## Linux & Mac

To use run:

`cargo build --release`

Then add the binary to your $PATH, for example:

`cp target/release/rusty-pullauta /usr/local/bin/`


For the script to work you may need to install the perl script dependencies:

`sudo cpan force install GD POSIX Config::Tiny Geo::ShapeFile`

Finally you'll also need the las2txt binary that you'll have to compile:

```
git clone https://github.com/LAStools/LAStools
cd LAStools
make
cp bin/las2txt /usr/local/bin/
```


Finally run the perl script as you would run the pullautin.exe, it will invoke the rust binary when posible, eg: 

`perl pullauta L3323H3.laz`


## Windows

On windows, you also need to compile the rust binary and put it in your path (same location where you put las2txt.exe), after that you can use the pullauta.exe file as you where using the original pullauta.exe file.