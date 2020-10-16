#!/bin/sh

# remove old build
rm -R ./public

# build site
HUGO_DISABLELANGUAGES="ar" hugo

# upload site to web server
rsync -azzhe "ssh -p 2223" ./public/ admin@qaul.net:/home/admin
