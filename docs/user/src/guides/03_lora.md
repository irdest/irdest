# LoRa radio setup

[LoRa]() is a low-power long-range wireless communication standard.
This guide will walk you through setting up two LoRa modems and
running a Ratman connection over it.  Following is the hardware list
you will need:

| Item                                  | Description                      | Link                                                                                                 | Alternatives |
|---------------------------------------|----------------------------------|------------------------------------------------------------------------------------------------------|--------------|
| STMicroelectronics NUCLEO-F401RE      | STM32 micro-controller dev board | https://eu.mouser.com/ProductDetail/STMicroelectronics/NUCLEO-F401RE?qs=fK8dlpkaUMvGeToFJ6rzdA%3D%3D |              |
| Seeed Studio Dragino Lora Shield 868m | LoRa wireless shield             | https://eu.mouser.com/ProductDetail/Seeed-Studio/114990615?qs=GZwCxkjl%252BU02ODDBHQ6wrw%3D%3D       |              |


## WARNING

The LoRa frequency band is allocated to users **per region**!  If you
violate your region's frequency standards **you may run into trouble
with the cops!** Please check and double-check that the frequency band
that Ratman is configured to is **correct for your region!**

Please also note that tuning Ratman broadcast parameters past legal
limits is not supported and the Irdest project is not responsible for
misconfigurations on your system!

## Frequency Setup

Please check your frequency allocations [here](wikipedia-whatever)...


## Building/ flashing firmware


## Configuring Ratman

