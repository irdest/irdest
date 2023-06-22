# Ratman client & interface library

Ratman is a packet router daemon, which can either run stand-alone, or
be embedded into existing applications.  This library provides type
definitions, utilities, and interfaces to interact with the Ratman
router core.

This library can be used in two different ways (not mutually
exclusive, although doing both at the same time would be a bit weird.
But hey, we won't judge you).

- You want to write a ratman-client application (i.e. a program that
uses Irdest as its network backend).  Use the types and functions
exported from the [client](crate::client) module
- You want to write a ratman-netmod driver (i.e. a plug-in for Ratman
to peer with other instances via some new communication channel).  Use
the types and functions exported from the [netmod](crate::netmod)
module

