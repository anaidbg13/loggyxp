#!/bin/bash

testDir="/tmp/dummyLogs"
mkdir -p "$testDir"
testFile="$testDir/demo.txt"
testFile2="$testDir/demo2.txt"
count=0
echo > "$testFile"

while true ; do 
    echo "Counting file 1 $count" >> "$testFile"
    echo "Counting file 2 $count" >> "$testFile2"
    count=$((count + 1)) 
    echo "Sleep for $1 sec"
    sleep $1
done 

