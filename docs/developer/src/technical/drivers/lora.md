# LoRa broadcast driver

The `netmod-lora` endpoint uses the `lora-modem` embedded firmware in the background to communicate with the actual radio hardware.  Check the user manual on how to set this up.

Irdest LoRa implements its own framing mechanism, incompatible to LoRaWAN.  We do set the magic number equivalent bit from LoRaWAN to a different identifier though, which means that LoRaWAN devices *should* ignore Irdest frames.

LoRa is a broadcast backplane, which means that unicasts can only be implemented via filtering.

More documentation to follow.  For more in-progress notes check out [this wiki page](https://hedgedoc.irde.st/i4HoJwh-S7ODRsytRqMBHQ)!
