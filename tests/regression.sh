#! /bin/bash

# Get the directory of the currently executing script
SCRIPT_DIR=$(dirname "$(realpath "$0")")

# Change the current directory to the script's directory
cd "$SCRIPT_DIR" || exit 1  # Exit if changing the directory fails

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
    fi
fi

cd data

echo "\n############ Running the current build of pullauta ############]/n"

WORK_DIR="CurrentBranch"
if [ ! -d "$WORK_DIR" ]; then
    echo "Directory does not exist. Creating..."
    mkdir -p "$WORK_DIR"
    if [ $? -eq 0 ]; then
        echo "Directory created: $WORK_DIR"
    else
        echo "Failed to create directory."
    fi
else
    echo "Directory already exists: $WORK_DIR"
    echo "Clearing contents of directory: $WORK_DIR"
    rm -rf "$WORK_DIR"/*
    if [ $? -eq 0 ]; then
        echo "Contents cleared from $WORK_DIR."
    else
        echo "Failed to clear contents of $WORK_DIR."
    fi
fi

cd "$WORK_DIR"

../../../target/release/pullauta ../test_file.laz

cd ..

echo "\n############ Running the latest release of pullauta ############]/n"

RELEASE_DIR="Release"
if [ ! -d "$RELEASE_DIR" ]; then
    echo "Directory does not exist. Creating..."
    mkdir -p "$RELEASE_DIR"
    if [ $? -eq 0 ]; then
        echo "Directory created: $RELEASE_DIR"
    else
        echo "Failed to create directory."
    fi
else
    echo "Directory already exists: $RELEASE_DIR"
fi

cd "$RELEASE_DIR"

LATEST_TAG=$(curl -sL https://api.github.com/repos/rphlo/karttapullautin/releases/latest | jq -r ".tag_name")
if [ ! -d "$LATEST_TAG" ]; then
    echo "Directory does not exist. Creating..."
    mkdir -p "$LATEST_TAG"
    if [ $? -eq 0 ]; then
        echo "Directory created: $LATEST_TAG"
        cd "$LATEST_TAG"
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
        curl -L "https://github.com/rphlo/karttapullautin/releases/latest/download/$FILE_NAME" | tar xvz
        ./pullauta ../../test_file.laz
        cd ..
    else
        echo "Failed to create directory."
    fi
else
    echo "Directory already exists: $LATEST_TAG"
fi


cd ..

echo "\n############ Comparing the outputs ############]n"

pngcomp "$WORK_DIR/pullautus.png" "$RELEASE_DIR/$LATEST_TAG/pullautus.png" | tee -a pngcomp.txt
pngcomp "$WORK_DIR/pullautus_depr.png" "$RELEASE_DIR/$LATEST_TAG/pullautus_depr.png" | tee -a pngcomp_depr.txt
