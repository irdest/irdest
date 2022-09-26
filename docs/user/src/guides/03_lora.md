# LoRa radio setup

[LoRa]() is a low-power long-range wireless communication standard.
This guide will walk you through setting up two LoRa modems and
running a Ratman connection over it.  Following is the hardware list
you will need:

| Item                                                                                                                                     | Description                              | Notes                                                                                                                                                                                               |
|------------------------------------------------------------------------------------------------------------------------------------------|------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| [ST Microelectronics NUCLEO-F401RE](https://eu.mouser.com/ProductDetail/STMicroelectronics/NUCLEO-F401RE?qs=fK8dlpkaUMvGeToFJ6rzdA%3D%3D) | STM32 micro controller development board | Currently only this board and CPU is supported, there is no reason for this selection other than it being available during the parts shortage at the time we ordered the parts. The firmware should be easily adaptable to any device that is supported by the [Rust Embedded project](https://github.com/rust-embedded). The only hard requirements are 1 Serial port and one SPI interface. |
| [Seeed Studio Dragino Lora Shield 868m](https://eu.mouser.com/ProductDetail/Seeed-Studio/114990615?qs=GZwCxkjl%252BU02ODDBHQ6wrw%3D%3D)  | LoRa wireless shield - Arduino Uno Formatted. | This module was selected as it fits directly on our dev board making development much easier. The module listed here is tuned for the European frequency band. Care must be taken to purchase the appropriate module for your region, more info below. The firmware supports any sx127 series modem, which is available on a wide range of hats, shields etc.                    |


## A Very Serious Warning About Working With Radios
**LoRa is a low level radio protocol. Unlike most common WiFi and Bluetooth modules LoRa radios will not hold your hand or stop you from getting yourself into trouble.**

LoRa exists in the ISM band of the radio spectrum, this band is regulated such that a user does not require a standard broadcast or operator licence to transmit. 

Because there are no strict training requirements for operating LoRa equipment there is the dangerous possibility that an uninformed operator could begin transmitting without knowledge of their legal requirements. Among other things the LoRa band has regional restrictions on:

  * How much power you can transmit with
  * What equipment you can use to transmit
  * How frequently and at what length you can transmit for (duty cycle)
  * Which frequencies you can transmit on.
  * Some Jurisdictions may also have restrictions on what content can be transmitted.

Most Nations are very vigilant about how their radio spectrum is used, and have deployed a network of listening stations to enforce breaches of radio law. Sadly breaches are often considered to be matters of national security, and the law will not likely be lenient on a hacker playing with a radio. 

If you violate broadcasting laws repercussions can include:
 
 * Hefty Fines (in the order of 10's of thousands of €/$/£'s)
 * Court appearances, and a criminal record
 * Confiscation of any radio equipment, and probation terms which prevent acquiring more.
 * Revocation of any radio licences you may hold
 * In extreme cases, or repeated offences, jail time.
 * Irdest additionally transmits encrypted data, which means if you give the authorities a reason to you may be charged with Espionage, Terrorism, or Computer Misuse. 

Any and all of these have life changing side effects, most severe of which is you'll likely loose whatever cushy tech job you currently work now, and won't be able to get another one. If you're charged with Computer Misuse in the US, you're not touching a computer again for 10 years. If you're charged with something worse... that doesn't bear writing in a readme.

### First thing's first. Irdest is a research project, do not deploy this.
Irdest is a not ready for deployment, it's LoRa components do not implement any guarantees of compliance with any regions laws, it particularly egregiously violates the duty cycle requirements. Irdest currently is only suitable for use in a laboratory environment for short term usage.

Naturally Irdest comes with no warranties of merchantability, fitness for use, or even being legal to operate. We will not be held responsible if you are found in violation of your local radio laws though using our hardware or software (which you will be, and you will be caught)

### Frequencies
Different parts of the world slice up the radio spectrum for different users differently. Because of this LoRa exists at different frequency bands:

  * 868Mhz (Europe)
  * 915Mhz (North America)
  * 486Mhz (China)
  * Others depending on region. 

Your equipment and software must both be configured for the correct frequency band. Very unfun things happen when you do not do this. For instance in Europe the American LoRa band (915MHz) overlaps with the European emergency services backup frequencies. Transmitting on these frequencies is a severe criminal offence. 

### Duty Cycle
LoRa's use of the ISM band is stipulated under the condition that no site (transmitter) will actively transmit for more than 1% of the time. This is calculated as no more than 36 seconds in any 1 hour window. Violation of this requirement can cause harmful interference with other users of the LoRa network. Keep in mind that LoRa can have a range of up to 10Km, so you likely will not know of this interference until a police officer comes to tell you to stop. **Irdest will violate this 1% requirement and likely interfere with other LoRa users**

## Setup
OK so you're mad enough to try this:
### 1. Collect equipment and prepare environment
The BoM table above specifies the parts for one station. For a useful setup you'll need at least two. In addition you'll also need a mini-USB cable and a separate computer for each station. Both Ratmand and the radio's debug bridge can only be run once on a computer, so you really will need two computers, sorry.

Assembling the radio components is easy. press the radio shield into the arduino connector on the CPU dev board. Next three jumpers SV2, SV3, SV4 need to be switched such that they connect the two pins toward the back of the dev board, furthest from the antenna connector, on each jumper respectively. Finally connect the antenna. 

Make sure you select small low gain antenna for a lab setup, and where possible position your lab as low as possible in the world. Underground is best. If you're particularly paranoid you can put your test setup in a shielded box or Faraday cage.

### 2. Go read the warning about radios section (yes again!)
**Reminder:** Irdest is not yet compliment with all LoRa radio requirements, only build a test environment low down, with low power antennae, and operate it for as short as needed to run your test.

### 3. Prepare the software.
The FREQUENCY field in `<repo_root>/firmware/lora-modem/src/main.rs` needs to be set for your region.

 * 868 for Europe
 * 915 for North America

Anywhere else contact us and we'll help you figure it out, cos it's not always clear how the different ISM bands map onto this one i64 value from our driver crate provider.

You may also want to consider reducing the frequency of announcements, as that is hard coded into the firmware.

*How?*

### 4. Configure Ratman
First ensure Ratmand has been run (and shut down) at least once, so that config files can be generated.

Next plug your dev board in and search for it's serial port path. It should be in the form of `/dev/ttyACMx` where x is a number.

modify your `~/.config/ratmand/config.json` to include 

```json
  "lora_port": "/dev/ttyACMx", #your path here
  "lora_baud": 9600,
  "no_lora": false,
```

### 5. Build software
To build you will need to install the arm thumbv7em-none-eabihf target, the easiest way to do this is to run:
```bash
rustup target install thumbv7em-none-eabihf
```
You will also need to install arm-none-eabi-gdb. This is packaged on Ubuntu and Arch, there's a COPR on Fedora.

The following steps need to be run from the firmware directory, so switch into `firmware/lora-modem`. 

in one terminal run `sudo openocd` (also needs to be in the firmware directory) This will open the debug port the the micro-controller and start a socket for gdb to connect to. Your dev board should now have a rapidly flashing green/red LED on it.

Next in a second terminal run `cargo run --release` this will compile the software and start a gdb session which will upload the firmware to the micro-controller. The release flag is always required due to the very tight timing requirements faced by processing serial data on a micro-controller.

After a few moments for the upload to complete you will be met with at `(gdb)` prompt. The micro-controller's CPU is halted at the beginning of main and ready to start. type `c` followed by enter to start the firmware.

### 6. Start Ratman
For testing we disable the inet driver, start Ratman with the following call:
```bash
cargo run --features lora --bin ratmand -- --no-inet --no-discovery -v trace
```

### 7. Testing

*TODO*

## Demonstration

Following is a video demonstrating how you can send messages through
`ratcat` via the LoRa link.

<iframe title="LoRa Irdest Demo (September 2022)" src="https://diode.zone/videos/embed/bccc81ae-495f-409f-86df-97c5b64e8c98" allowfullscreen="" sandbox="allow-same-origin allow-scripts allow-popups" width="560" height="315" frameborder="0"></iframe>
