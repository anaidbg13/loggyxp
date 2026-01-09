#!/bin/bash

testDir="/tmp/dummyLogs"
mkdir -p "$testDir"
testFile="$testDir/demo.txt"
count=0
echo > "$testFile"

while true ; do 
    echo "Counting bbbbb $count" >> "$testFile"
    count=$((count + 1)) 
    echo "Sleep for $1 sec"
    sleep $1
done 

