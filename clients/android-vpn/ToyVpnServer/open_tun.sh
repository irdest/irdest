#!/usr/bin/env sh

# This script configures the TUN interface on the linux system.

# Change belows as you wish.
TUN_NAME="tun0"
SRC_ADDR="10.0.0.1/32"
DEST_ADDR="10.0.0.2/32"

echo "🐝 Setting up the tun interface \n
    | name = $TUN_NAME | source address = $SRC_ADDR | destination address = $DEST_ADDR |"

echo 1 > /proc/sys/net/ipv4/ip_forward

ip tuntap add name $TUN_NAME mode tun
ip addr add $SRC_ADDR peer $DEST_ADDR dev $TUN_NAME
ip link set $TUN_NAME up

iptables -t nat -A POSTROUTING -s $DEST_ADDR -j MASQUERADE
iptables -A FORWARD -i $TUN_NAME -s $DEST_ADDR -j ACCEPT
iptables -A FORWARD -o $TUN_NAME -s $DEST_ADDR -j ACCEPT

echo "Done."
