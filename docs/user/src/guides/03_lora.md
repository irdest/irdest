# LoRa radio setup

[LoRa]() is a low-power long-range wireless communication standard.
This guide will walk you through setting up two LoRa modems and
running a Ratman connection over it.  Following is the hardware list
you will need:

| Item                                                                                                                                     | Description                              | Alternatives                                                                                                                                                                                               |
|------------------------------------------------------------------------------------------------------------------------------------------|------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| [STMicroelectronics NUCLEO-F401RE](https://eu.mouser.com/ProductDetail/STMicroelectronics/NUCLEO-F401RE?qs=fK8dlpkaUMvGeToFJ6rzdA%3D%3D) | STM32 micro controller development board | Any STM32 micro-controller _should_ work, but please beware that you will have to compile your own firmware if the memory layout of the model you choose is different than what we used during development |
| [Seeed Studio Dragino Lora Shield 868m](https://eu.mouser.com/ProductDetail/Seeed-Studio/114990615?qs=GZwCxkjl%252BU02ODDBHQ6wrw%3D%3D)  | LoRa wireless shield                     |                                                                                                                                                                                                            |


## WARNING

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
