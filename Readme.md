# Rusty Kartta Pullautin

Work In Progress

Currently blocks, makecliffs, makevegenew and xyz2contour steps are translated to rust, however they can already be used within the full process.

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
