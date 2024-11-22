#! /bin/bash

usage="$(basename "$0") [-h] [-v latest] [-p pullauta] -- test the current build of pullauta (assumed to be in target/release/) \
against the latest release

where:
	-h  show this help text
	-v  use a specific release version
	-p  location of pullauta executable to test
	-i  location of the pullauta.ini file to use
"

VERSION="latest"
PULLAUTA="auto"
INI="auto"
while getopts "hp:v:i:" opt; do
	case $opt in
		h) echo "$usage"
		exit
		;;
		v) VERSION="$OPTARG"
		;;
		p) PULLAUTA="$OPTARG"
		if [ ! -e "$PULLAUTA" ] || [ ! -x "$PULLAUTA" ]; then
			echo "The file $PULLAUTA either does not exist or is not executable."
			exit 1
		else
			PULLAUTA=$(realpath "$PULLAUTA")
		fi
		;;
		i) INI="$OPTARG"
		if [ ! -e "$INI" ] || [ "$INI" == *.ini ]; then
			echo "The file $INI either does not exist or does not have the correct extension."
			exit 1
		else
			INI=$(realpath "$INI")
		fi
		;;
		\?) echo "Invalid option -$OPTARG" >&2
		exit 1
		;;
	esac

	case $OPTARG in
		-*) echo "Option $opt needs a valid argument"
		exit 1
		;;
	esac
done

if [[ "$VERSION" != "latest" ]]; then
	read -ra RELEASES <<< $(curl https://api.github.com/repos/rphlo/karttapullautin/releases | jq '.[] | .tag_name'  | tr -d '"')
	IFS=' '
	# Flag for finding the element
	found=false
	# Loop through the array
	for element in "${RELEASES[@]}"; do
		if [[ "$element" == "$VERSION" ]]; then
			found=true
			break
		fi
	done

	# Output the result
	if [[ "$found" == false ]]; then
		echo "$VERSION is not a valid version."
		exit 1
	fi
fi

# Get the directory of the currently executing script
SCRIPT_DIR=$(dirname "$(realpath "$0")")

# Change the current directory to the script's directory
cd "$SCRIPT_DIR" || exit 1  # Exit if changing the directory fails
if [[ "$PULLAUTA" == "auto" ]]; then
	PULLAUTA=$(realpath "../target/release/pullauta")
fi

# Print the current directory to confirm
echo "Current directory is: $(pwd)"

DIR="data"
if [ ! -d "$DIR" ]; then
	echo "Directory does not exist. Creating..."
	mkdir -p "$DIR"
	if [ $? -eq 0 ]; then
		echo "Directory created: $DIR"
	else
		echo "Failed to create directory."
		exit 1
	fi
else
	echo "Directory already exists: $DIR"
fi

FILE="data/test_file.laz"
if [ ! -f "$FILE" ]; then
	curl -L "https://cdn.routechoic.es/test.laz" -o "$FILE"
	if [ $? -eq 0 ]; then
		echo "Download complete: $FILE"
	else
		echo "Failed to download file."
		exit 1
	fi
fi

cd data

echo -e "\n############ Running the current build of pullauta ############\n"

WORK_DIR="CurrentBranch"
if [ ! -d "$WORK_DIR" ]; then
	echo "Directory does not exist. Creating..."
	mkdir -p "$WORK_DIR"
	if [ $? -eq 0 ]; then
		echo "Directory created: $WORK_DIR"
	else
		echo "Failed to create directory."
		exit 1
	fi
else
	echo "Directory already exists: $WORK_DIR"
	echo "Clearing contents of directory: $WORK_DIR"
	rm -rf "$WORK_DIR"/*
	if [ $? -eq 0 ]; then
		echo "Contents cleared from $WORK_DIR."
	else
		echo "Failed to clear contents of $WORK_DIR."
		exit 1
	fi
fi
WORK_DIR=$(realpath "$WORK_DIR")

cd "$WORK_DIR"
if [ "$INI" != "auto" ]; then
	cp $INI "pullauta.ini"
fi
"$PULLAUTA" ../test_file.laz

cd ..

echo -e "\n############ Running the $VERSION release of pullauta ############\n"

RELEASE_DIR="Release"
if [ ! -d "$RELEASE_DIR" ]; then
	echo "Directory does not exist. Creating..."
	mkdir -p "$RELEASE_DIR"
	if [ $? -eq 0 ]; then
		echo "Directory created: $RELEASE_DIR"
	else
		echo "Failed to create directory."
		exit 1
	fi
else
		echo "Directory already exists: $RELEASE_DIR"
fi

RELEASE_DIR=$(realpath "$RELEASE_DIR")
cd "$RELEASE_DIR"

if [[ "$VERSION" == "latest" ]]; then
	TAG=$(curl -sL https://api.github.com/repos/rphlo/karttapullautin/releases/latest | jq -r ".tag_name")
else
	TAG=$VERSION
fi
if [ ! -d "$TAG" ]; then
	echo "Directory does not exist. Creating..."
	mkdir -p "$TAG"
	if [ $? -eq 0 ]; then
		echo "Directory created: $TAG"
		cd "$TAG"
		# Detect the operating system
		OS=""
		if [[ "$OSTYPE" == "linux-gnu"* ]]; then
			OS="linux"
		elif [[ "$OSTYPE" == "darwin"* ]]; then
			OS="macos"
		elif [[ "$OSTYPE" == "cygwin" || "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
			OS="win"
		else
			echo "Unsupported operating system: $OSTYPE"
			exit 1
		fi
		# Detect the architecture
		ARCH=""
		if [[ "$(uname -m)" == "x86_64" ]]; then
			ARCH="x86_64"
		elif [[ "$(uname -m)" == "aarch64" ]]; then
			ARCH="arm64"
		elif [[ "$(uname -m)" == "arm64" ]]; then
			ARCH="arm64"
		else
			echo "Unsupported architecture: $(uname -m)"
			exit 1
		fi
		FILE_NAME="karttapullautin-${ARCH}-${OS}.tar.gz"

		if [[ "$VERSION" == "latest" ]]; then
			URL="https://github.com/rphlo/karttapullautin/releases/latest/download/$FILE_NAME"
		else
			URL="https://github.com/rphlo/karttapullautin/releases/download/$TAG/$FILE_NAME"
		fi

		curl -L $URL | tar xvz

		if [ "$INI" != "auto" ]; then
			cp $INI "pullauta.ini"
			echo "$INI" > "whatInit.txt"
		else
			echo "auto" > "whatInit.txt"
		fi
		./pullauta ../../test_file.laz
		cd ..
	else
		echo "Failed to create directory."
	fi
else
	echo "Directory already exists: $TAG"
	cd $TAG
	if [ "$INI" != "auto" ]; then
		if ! diff <(grep -Ev '^\s*$|^\s*#' "$INI") <(grep -Ev '^\s*$|^\s*#' "pullauta.ini") > /dev/null; then
			cp $INI "pullauta.ini"
			echo "$INI" > "whatInit.txt"
			./pullauta ../../test_file.laz
		fi
	else
		if [ "$(<"whatInit.txt")" != "auto" ]; then
			echo "auto" > "whatInit.txt"
			rm "pullauta.ini"
			./pullauta ../../test_file.laz
		fi
	fi
	cd ..
fi

cd ..

echo -e "\n############ Comparing the outputs ############\n"

CURRENT="Results-$(date +'%Y%m%d_%H%M%S')"
mkdir "$CURRENT"
cd "$CURRENT"

cp "$WORK_DIR/pullauta.ini" "pullauta.ini"
printf "%s\n" ".ini file: $INI" >> "specs.txt"
printf "%s\n" "release: $TAG" >> "specs.txt"
printf "%s\n" "commit: $(git rev-parse --short HEAD)" >> "specs.txt"
printf "%s\n" "status:" >> "specs.txt"
printf "%s\n" "$(git status -s)" >> "specs.txt"

if command -v pngcomp &> /dev/null; then
	pngcomp "$WORK_DIR/pullautus.png" "$RELEASE_DIR/$TAG/pullautus.png" | tee -a "pngcomp_$TAG.txt"
	pngcomp "$WORK_DIR/pullautus_depr.png" "$RELEASE_DIR/$TAG/pullautus_depr.png" | tee -a "pngcomp_depr_$TAG.txt"
else
	echo "Comparison failed. Please install the pngnq package."
	exit 1
fi

if command -v magick &> /dev/null; then
	magick compare "$WORK_DIR/pullautus.png" "$RELEASE_DIR/$TAG/pullautus.png" "pullautus_comp_$TAG.png"
	magick compare "$WORK_DIR/pullautus_depr.png" "$RELEASE_DIR/$TAG/pullautus_depr.png" "pullautus_comp_depr_$TAG.png"
else
	echo "Comparison failed. Please install the imagemagick package."
	exit 1
fi

