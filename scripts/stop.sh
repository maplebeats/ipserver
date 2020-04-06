#!/bin/bash
ps aux|grep release/ipserver|grep -v grep|awk '{print $2}'|xargs kill