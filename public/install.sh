#!/bin/sh

BUILD=$1

if [ -z "$BUILD" ]; then
	echo "Usage: <build>"
	exit 1
fi

if ! [ "$BUILD" -eq "$BUILD" ] 2> /dev/null; then
	echo "Build must be numeric"
	exit 1
fi

DATA=$(curl -s -H "Accept: application/json" https://mc.rjns.dev/api/v1/build/$BUILD)

if echo $DATA | grep -q '"success":false'; then
	echo "Build $BUILD not found"
	exit 1
fi

TYPE=$(echo $DATA | sed -n 's/.*"type":"\([^"]*\)".*/\1/p')
VERSION_ID=$(echo $DATA | sed -n 's/.*"versionId":"\([^"]*\)".*/\1/p')
PROJECT_VERSION_ID=$(echo $DATA | sed -n 's/.*"projectVersionId":"\([^"]*\)".*/\1/p')
BUILD_NUMBER=$(echo $DATA | sed -n 's/.*"buildNumber":\([0-9]*\).*/\1/p')
JAR_URL=$(echo $DATA | sed -n 's/.*"jarUrl":"\([^"]*\)".*/\1/p')
JAR_LOCATION=$(echo $DATA | sed -n 's/.*"jarLocation":"\([^"]*\)".*/\1/p')
ZIP_URL=$(echo $DATA | sed -n 's/.*"zipUrl":"\([^"]*\)".*/\1/p')

if [ "$VERSION_ID" = "null," ]; then
	VERSION_ID=""
fi
if [ "$PROJECT_VERSION_ID" = "null," ]; then
	PROJECT_VERSION_ID=""
fi
if [ "$JAR_URL" = "null," ]; then
	JAR_URL=""
fi
if [ "$JAR_LOCATION" = "null," ]; then
	JAR_LOCATION="server.jar"
fi
if [ "$ZIP_URL" = "null," ]; then
	ZIP_URL=""
fi

echo "Install MCVAPI build $BUILD into current location?"
echo ""

echo "Type: $TYPE"
if [ -n "$VERSION_ID" ]; then
	echo "Minecraft Version: $VERSION_ID"
fi
if [ -n "$PROJECT_VERSION_ID" ]; then
	echo "Project Version: $PROJECT_VERSION_ID"
fi

if [ "$BUILD_NUMBER" -ne 1 ] || [ -z "$PROJECT_VERSION_ID" ]; then
	echo "Build Number: $BUILD_NUMBER"
fi

echo ""
echo "[y]es or [n]o"
read -r CONFIRM

if [ "$CONFIRM" != "y" ] && [ "$CONFIRM" != "yes" ]; then
	echo "Installation cancelled"
	exit 1
fi

if [ -d "libraries" ]; then
	rm -rf libraries
fi

if [ -n "$JAR_URL" ]; then
	echo "Downloading $JAR_URL to $JAR_LOCATION..."
	curl -s -o $JAR_LOCATION $JAR_URL
	echo "Downloading $JAR_URL to $JAR_LOCATION... Done"
fi

if [ -n "$ZIP_URL" ]; then
	echo "Downloading $ZIP_URL..."
	curl -s -o server.zip $ZIP_URL
	echo "Downloading $ZIP_URL... Done"

	echo "Extracting server.zip..."
	unzip -q server.zip
	echo "Extracting server.zip... Done"

	rm server.zip
fi

echo "java -Xmx4G -Xms4G -jar server.jar nogui" > start.sh
chmod +x start.sh

echo "Installation complete"
echo ""

echo "To start the server, run ./start.sh"
echo "To stop the server, type 'stop' in the console or press Ctrl+C"
echo "To configure the server, edit eula.txt and server.properties"
echo "To update the server, run this script with the new build number"
