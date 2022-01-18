# Available clients

The Irdest project provides several applications for different
use-cases.  At the center is Ratman, a decentralised router daemon.
Ratman also comes with a set of tools, each documented in their own
section of this manual.

But Ratman itself is only a tool to enable other applications to
interact with a decentralised mesh network.  As such, Irdest also
tries to provide a set of example applications and tools that can
demonstrate the usability of this network.

However, so far there are no graphical Irdest applications.


| Client name   | Description                                                     | Version |
|---------------|-----------------------------------------------------------------|---------|
| [ratmand]     | Stand-alone router daemon                                       | 0.3.0   |
| [ratcat]      | Similar to `netcat`, but using Ratman as a network backend      | 0.3.0   |
| [ratctl]      | Configuration and utility CLI for Ratman daemon                 | 0.3.0   |
| [irdest-echo] | A simple demo application which echoes all messages it receives | 0.1.0   |

[ratmand]: ./ratman/ratmand.md
[ratcat]: ./ratman/ratcat.md
[ratctl]: ./ratman/ratctl.md
[irdest-echo]: ./irdest-echo.md
