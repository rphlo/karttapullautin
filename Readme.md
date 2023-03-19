# Rusty Kartta Pullautin

Work In Progress

Currently only xyz2contour is translated to rust.

To use run:

`cargo build --release`

Then add the binary to your $PATH for example:

`cp target/release/rusty-pullauta /usr/local/bin`

Finally run the perl script as usual, it will invoke the rust binary when posible, eg: 

`perl pullauta L3323H3.laz`
