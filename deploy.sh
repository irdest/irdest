#!/bin/sh

# remove old build
rm -R ./public

# build site
hugo

# upload site to web server
rsync -azhe "ssh -p 2223" ./public/ admin@qaul.net:/home/admin
