#!/bin/bash

echo "Kill Xvfb"
killall -9 Xvfb

rm /tmp/.X42-lock
rm -rf /tmp/*

echo "Wait before create Xvfb"
sleep 3

nohup Xvfb :42 -screen 0 1920x1024x16 &

echo "Xvfb pid: "
echo $!

echo "Wait after crate Xvfb"
sleep 5

export DISPLAY=:42
#DISPLAY=:42 ./news_parser

# nohup ./news_server > server_LOG &
./news_server &>> server_LOG