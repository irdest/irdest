# Ratcat

A tool similar to `netcat` that allows you to interact with a Ratman
network from various scripts and small programs.

Before you can send messages you need to call `ratcat` with the
`--register` flag.  This will register a new address with your local
router.  Afterwards you can send and receive messages.


## Sending messages

`ratcat` will read a message either from the commandline arguments or
from standard input.  In either case, the first required parameter is
_always_ a recipient.

A recipient is a Ratman address, which has the following format:

```
ECB4-30B9-4416-C403-716F-601F-FC56-9AD3-BD2E-3892-227A-84AD-E6FC-A1CE-0A92-03F6
```


So to send a short message to this address you would call:

```console
$ echo "Hello ECB4... do you have a nickname?" | ratcat 'ECB4-30B9-4416-C403-716F-601F-FC56-9AD3-BD2E-3892-227A-84AD-E6FC-A1CE-0A92-03F6'
```


## Receiving messages

`ratcat` can also be set to receive messages.  For this simply provide
the `--recv` flag.  This will wait forever and print messages (or to
standard output if a pipe is connected).  You can limit this behaviour
with `--count` which takes a number as argument.

For example, the following invocation will wait for an incoming
message and then pipe the resulting output into `json_pp`.  Any errors
or warnings are printed to standard error.

```console
$ ratcat --recv --count 1 | json_pp
```


You can also combine message sending with receiving (simply call
`--recv` after the recipient/ message info)


## State

`ratcat` stores your last registered address in:

- Linux/ BSD/ XDG system: `$XDG_CONFIG_HOME/ratcat/config`.
- macOS: `/Users/[USER_NAME]/Library/Application Support/org.irdest.ratcat`.
