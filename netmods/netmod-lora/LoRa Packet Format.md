# Lora Packet format.
Due to the limitations of the radio complete packets need to be at most 255 Bytes
Due to the existence of other protocols on the network, we will need a header format to identify irdest traffic.

To avoid collisions with LoRaWAN which does not specify a protocol identification magic number irdest will use the protocol identification code of 0xCA. This is chosen to match the RFU Frame type field in LoRaWAN. This will hopefully prevent LoRaWAN devices from being confused by our frames. 

The second byte contains the control packet format. The following control packet types exist with their payloads:

 - 0x00: invalid packet
 - 0x01: data packet: payload - irdest frame
 - 0x02: retransmit request - no payload, length feild = 0

The next byte will contain the length of the payload this is the length of the irdest block used. The payload immediately follows.

The final 4 bytes are reserved for a CRC checksum, however in the proof of concept phase this feild will be filled with 0's

Frame layout:
```
[magic][type][length][--------------payload------------]
0      1     2       3                             len+2

```

LAP-B????????????????

The default payload length will be 252 bytes. Making the default packet length the full 255. 

The Modem will attempt to read idest packets in 128 byte blocks. On completing a 128 byte transfer the data will padded with the frame format above, and transmitted. 

On reception of any message the modem will check the magic number.
if this is valid the payload will be extracted and submitted over serial.