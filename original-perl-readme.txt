
Karttapullautin   (c) Jarkko Ryyppo 2012-2013  All rights reserved.

This exe is free for non commercial use or if used for for navsport 
mapping (orienteering, rogaining, adventure racing mapping).
There is no warranty. Use it at your own risk!

________________________________________

(Ohjeet suomeksi tiedoston loppuosassa)

________________________________________

Basic use

Karttapullautin accepts xyz file with classification (xyzc). las2txt from lastools
http://www.cs.unc.edu/~isenburg/lastools/

* If las2txt.exe is placed in the same folder as pullauta.exe, Karttapullautin can
make the conversion for you, so you can just drop las/laz file on pullautin.exe or type command 
pullauta yourlazfile.laz

You can also make fin advance from command line:

las2txt -i yourinputfile.laz -parse xyzcnri -o youroutputfile.xyz

Then just copy pullauta.exe and your xyz file to same folder and drag and drop xyz
file on pullauta.exe. Or alternatively form command line type shell command at that folder:
pullauta youroutputfile.xyz


As output Karttapullautin writes two 600 dpi png map images. One without depressions and
one with purple depressions. It also writes contours and cliffs as dxf files to 
temp folder to be post processed, for example using Open Orienteering Mapper or Ocad.

* You can re-render png map files (like with changed north line settings) by double clicking the exe file.

* For Finns: Karttapullautin can also render Maastotietokanta zip files (shape files) downloaded from
the download site of Maanmittauslaitos. After normal process just drag & drop Maastotietokanta zip(s) on 
the exe (you can drag several zip files at once). Or alternatively give shell command:
pullauta yourzipfile1.zip yourzipfile2.zip yourzipfile3.zip yourzipfile4.zip
Command re-renders png files with some Maastotietokanta vectors layers (roads, fields, buildings, water etc).
 
* To print a map at right scale, you download IrfanView http://www.irfanview.com/ 
open png map, Image -> Information, set resolution 600 x 600 DPI and push "change" button and save. 
Then crop map if needed (Select area with mouse and Edit -> crop selection). Print using "Print size: Original Size srom DPI". 
Like this your map should end up 1:10 000 scale on paper.

________________________________________

Fine tuning the output

Pullauta.exe creates pullauta.ini file if it doesn't alredy exists. Your settings are
there. For the second run you can change settings as you wish.
Experiment with small file to find best settings for your taste/terrain/lidar data.


Ini file configuration explanation, see ini file.

________________________________________

Re-processing steps again

When the process is done and you find there is too much green or too small cliffs, you can make parts 
of the process again with different parameters without having to do it all again. To re-generate only 
vegetation type from command line:

pullauta makevegenew
pullauta


To make cliffs again:
pullauta makecliffs xyztemp.xyz 1.0 1.15
pullauta
________________________________________

Vectors

In additon to the png raster map imges, Karttapullautin makes also vector contours and cliffs and also 
some raster vector files one might find intresting for mapping use. After the process you can find 
them in temp folder.

out2.dxf, final contours with 2.5 m interval
dotknolls.dxf, dot knolls and small U -depressions. Some are not rendered to png files for legibility reasons.
c1g.dxf, smal cliffs
c2g.dxf, big cliffs

vegetation.png + vegetation.pgw, generalized green/yellow as raster, same as at the background of final map png files.

For importing Maastotietokanta, try reading shape filed directly to your mapping app or try Pellervo K�ssi's 
excellent python script for converting and importing Maastotietokanta to Ocad http://koti.kapsi.fi/kassi/mtk2dxf/

________________________________________

BATCH PROCESSING

KArttapulautin can also batch process all las/las files + Maastotietokanta zips is a directory. 
To do it, turn batch processing on in ini file. configure your input file directory and output 
directory for map tiles. Copy your input files to input directory and double click pullauta.exe. 
It starts processing las/laz files one by one until everything is done. If you have several cores 
in your CPU, you can have several pullautin instances processing same files. you can configure it with 
"processes" parameter in ini file. Note, processes parameter effects only batch mode, in normal mode it uses 
just one worker process. You will also need lots of RAM to process simultaneously several large 
laser files. To re-process tiles in bach mode you need to remove previous png files from output folder.

You can merge png files in output folder with Karttapullautin. Without depressions versions with command line command

pullauta pngmerge 1

and depression versions

pullauta pngmergedepr 1

vegetation backround images (if saved, there is parameter for saving there)

pullauta pngmergevege


The last paramameter (number) is scale factor. 2 reduces size to 50%, 4 to 25%, 20 to 5% and so on. Command writes out jpg and png versions. 
Note, you easily run out of memory if you try merging together too large area with too high resolution.

You can also merge dxf files  (if saved, there is parameter for saving there)

pullauta dxfmerge

__________________________________________

Laser scanning point ground classification correction feature

You use Karttapullautin to find point classification erros and fix them. It's done like this:
- make map as usual (not in batch more)
- when it's done and look at the map, you may find place or places you are supposed to have hill or knoll but you have strange dark green. It's a sign of gound points classfied as something else (as vegetation/building/other).
- type from command line command 
pullauta ground
- it makes "ground.png" image, 1 m/pixel map of your map area, but just ground points. Open it with photo editor. Ground points are drawn there with black. If you have strange white areas at the place you saw that green, it means you have no ground points at all there, so all of your ground points are classified as something else. 
- To fix it, draw with red color areas you like to get ground classified again. No need to re-classify the whole data, just the tiny clip with the error. Save the ground.png image.
- type command line command
pullautta groundfix 3
- the "3" means here Karttapullautin will search the lowest hit for each 3x3 m you indicated with red. You can use other values than 3, take a look at your gound.png to figure out how much ground points you about have and type this value accordingly. Too small value may end up as spiky knolls, (some vegetation points classified as ground).  
- it writes out xyztempfixed.xyz. Re-process your map using this file, and you should get the area you indicated with red mapped a lot better. The rest of the map should remain the same.
