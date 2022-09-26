# LoRa radio setup

[LoRa]() is a low-power long-range wireless communication standard.
This guide will walk you through setting up two LoRa modems and
running a Ratman connection over it.  Following is the hardware list
you will need:

| Item                                                                                                                                     | Description                              | Notes                                                                                                                                                                                               |
|------------------------------------------------------------------------------------------------------------------------------------------|------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| [STMicroelectronics NUCLEO-F401RE](https://eu.mouser.com/ProductDetail/STMicroelectronics/NUCLEO-F401RE?qs=fK8dlpkaUMvGeToFJ6rzdA%3D%3D) | STM32 micro controller development board | Currently only this board and CPU is supported, there is no reason for this selection other than it being available during the parts shortage at the time we ordered the parts. The firmware should be easilly adaptable to any device that is supported by the [Rust Embedded project](https://github.com/rust-embedded). The only hard requirements are 1 Serial port and one SPI interface. |
| [Seeed Studio Dragino Lora Shield 868m](https://eu.mouser.com/ProductDetail/Seeed-Studio/114990615?qs=GZwCxkjl%252BU02ODDBHQ6wrw%3D%3D)  | LoRa wireless shield - Arduino Uno Formatted. | This module was selected as it fits directly on our dev board making development much easier. The module listed here is tuned for the European frequency band. Care must be taken to purchase the appropriate module for your region, more info below. The firmware supports any sx127 series modem, which is available on a wide range of hats, shields etc.                    |


## A VERY SERIOUS WARNING ABOUT WORKING WITH RADIOS

The LoRa frequency band is allocated to users **per region**!  If you
violate your region's frequency standards **you may run into trouble
with your local frequency management agency!** Please check and
double-check that the frequency band that Ratman is configured to is
**correct for your region!**

Please also note that tuning Ratman broadcast parameters past legal
limits is not supported and the Irdest project is not responsible for
misconfigurations on your system!

## Frequency Setup

Please check your frequency allocations [here](wikipedia-whatever)...


## Building/ flashing firmware


## Configuring Ratman


## Demonstration

Following is a video demonstrating how you can send messages through
`ratcat` via the LoRa link.

<iframe title="LoRa Irdest Demo (September 2022)" src="https://diode.zone/videos/embed/bccc81ae-495f-409f-86df-97c5b64e8c98" allowfullscreen="" sandbox="allow-same-origin allow-scripts allow-popups" width="560" height="315" frameborder="0"></iframe>
