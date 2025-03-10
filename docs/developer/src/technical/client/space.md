# Working with spaces

Spaces (also called 'namespaces' in places) are a way for many devices to receive the same set of messages.  They are analogues to a [multicast] address in IP networking.  They can be used to do a variety of things:

1. Let different instances of the same application find each other on the network
2. Compare the ping-times of different devices amongst each other (anycast)
3. Provide a more efficient mechanism to send the same data to multiple recipients

Spaces can either be long-term setups or created ad-hoc.  The only requirement is access to the _namespace keypair_, created by `ratctl space generate` on the commandline or `libratman::generate_space_key()` in code.  Messages to a namespace are still encrypted, but since multiple applications/ devices/ recipients share the same private key the sent messages should not be considered confidential.

## Creating and loading a space key

A space can be created through `libratman` on the fly, for example to create a group of multiple recipients.  This way you avoid encoding/ encrypting data multiple times to multiple recipients.  The created space key must first be sent to all group participants though.

```rust
// ... setup IPC connection as usual

let (pubkey, privkey) = libratman::generate_space_key();
ipc.space_load(pubkey, privkey).await?;
```

Alternatively you can create a spacekey via `ratctl` on the commandline (you can also pass the `-o json` flag on the cli root to switch to json output).

```console
$ ratctl space generate -f ./my-space
$ cat my-space
pubkey=4205-1618-129C-5AC8-B4E6-B1A1-491D-48AB-FE2C-FC68-9A5C-CB2A-F295-335F-71C1-A739
privkey=78B5-B4E3-0D74-CCB2-2087-3766-E546-5633-9C08-B206-5404-3602-6F9A-94FA-E3ED-A1B9
```

Building an application in Rust you can include this namespace keypair file in your application source to use across all instances:

```rust
let spacekey = include_str!("./my-space");

let (pubkey, privkey) = // parsing left as an excercise to the reader

ipc.space_load(pubkey, privkey).await?;
```
