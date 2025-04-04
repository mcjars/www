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

DATA=$(curl -s -H "Accept: application/json" https://versions.mcjars.app/api/v1/build/$BUILD)

if echo $DATA | grep -q '"success":false'; then
	echo "Build $BUILD not found"
	exit 1
fi

TYPE=$(echo $DATA | sed -n 's/.*"type":"\([^"]*\)".*/\1/p')
VERSION_ID=$(echo $DATA | sed -n 's/.*"versionId":"\([^"]*\)".*/\1/p')
PROJECT_VERSION_ID=$(echo $DATA | sed -n 's/.*"projectVersionId":"\([^"]*\)".*/\1/p')
BUILD_NUMBER=$(echo $DATA | sed -n 's/.*"buildNumber":\([0-9]*\).*/\1/p')

if [ "$VERSION_ID" = "null," ]; then
	VERSION_ID=""
fi
if [ "$PROJECT_VERSION_ID" = "null," ]; then
	PROJECT_VERSION_ID=""
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

if [ "$CONFIRM" != "y" ] && [ "$CONFIRM" != "yes" ] && [ "$CONFIRM" != "ye" ]; then
	echo "Installation cancelled"
	exit 1
fi

bash <(curl -s https://versions.mcjars.app/api/v1/script/$BUILD/bash)

echo "java -Xmx4G -Xms4G -jar server.jar nogui" > start.sh
chmod +x start.sh

echo "Installation complete"
echo ""

echo "To start the server, run ./start.sh"
echo "To stop the server, type 'stop' in the console or press Ctrl+C"
echo "To configure the server, edit eula.txt and server.properties"
echo "To update the server, run this script with the new build number"
