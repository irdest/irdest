# Stand-alone installer

A stand-alone way to install Ratman on a system.  The installer is
included in the Irdest bundle and statically linked, which means it
has no external dependencies.  Make sure that you download the bundle
for the correct CPU architecture (onlny 64bit x86 and 64bit ARM
available at the moment)!

## Unpack bundle

You can unpack the bundle with `tar` either on the terminal, or via
your graphical file browser.

```
$ tar xf Downloads/ratman-bundle-x86_64.tar.gz
$ cd Downloads/ratmand-bundle
$ ls
bin/  dist/  install*  man/  manual/  README.md
```

## Install or upgrade

If you already have a previous version of Ratman installed on your
system we recommend you uninstall it completely first:

```
$ ./uninstall
$ rm -r ~/.config/ratmand/ ~/.local/share/ratmand
```

Then run the installer

```console
$ ./install

  ██████╗  █████╗ ████████╗███╗   ███╗ █████╗ ███╗   ██╗
  ██╔══██╗██╔══██╗╚══██╔══╝████╗ ████║██╔══██╗████╗  ██║
  ██████╔╝███████║   ██║   ██╔████╔██║███████║██╔██╗ ██║
  ██╔══██╗██╔══██║   ██║   ██║╚██╔╝██║██╔══██║██║╚██╗██║
  ██║  ██║██║  ██║   ██║   ██║ ╚═╝ ██║██║  ██║██║ ╚████║
  ╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝   ╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝


This installer will determine how to install Ratman on your system!
(NEW) bin/ratmand -> /home/.local/bin/ratmand
(NEW) bin/ratcat -> /home/.local/bin/ratcat
(NEW) bin/ratctl -> /home/.local/bin/ratctl
(NEW) man/ratmand.1 -> /home/.local/share/man/man1/ratmand.1
(NEW) dist/ratman.service -> /home/.config/systemd/user/ratman.service
Do you want to proceed? (Y/n) 
Install /home/.local/bin/ratmand: OK
Install /home/.local/bin/ratcat: OK
Install /home/.local/bin/ratctl: OK
Install /home/.local/share/man/man1/ratmand.1: OK
systemctl daemon-reload: OK
Operation complete!
```

## Configuration and setup

The installer creates a service file for auto-starting Ratman.  But
this service is not enabled by default.  You can either run Ratman
manually on every start-up:

```
$ systemctl --user start ratman
```

Or you can setup auto-starting by "enabling" the service:

```
$ systemctl --user enable ratman
```

Verify that Ratman is running correctly:

```
$ systemctl --user status ratman
● ratman.service - A decentralised and peer-to-peer packet router
     Loaded: loaded (/home/.config/systemd/user/ratman.service; static)
     Active: active (running) since Wed 2022-10-05 20:19:14 CEST; 2s ago
   Main PID: 353991 (ratmand)
      Tasks: 18 (limit: 56248)
     Memory: 2.2M
        CPU: 6ms
     CGroup: /user.slice/user-1000.slice/user@1000.service/app.slice/ratman.service
             └─353991 /home/.local/bin/ratmand --accept-unknown-peers

Oct 05 20:19:14 theia systemd[3325]: Started A decentralised and peer-to-peer packet router.
Oct 05 20:19:14 theia ratmand[353991]: Oct 05 20:19:14.265  INFO ratman::daemon: Initialised logger: welcome to ratmand!
Oct 05 20:19:14 theia ratmand[353991]: Oct 05 20:19:14.265  INFO new{bind="[::]:9000" name="ratmand" mode=Dynamic}: netmod_inet: Initialising Tcp backend
Oct 05 20:19:14 theia ratmand[353991]: Oct 05 20:19:14.265  INFO netmod_inet::server: Listening on Ok([::]:9000) for incoming connections
Oct 05 20:19:14 theia ratmand[353991]: Oct 05 20:19:14.266  INFO ratmand: Auto-selected interface 'wlp3s0' for local peer discovery.  Is this wrong?  Pass --di>
Oct 05 20:19:14 theia ratmand[353991]: Oct 05 20:19:14.266  INFO netmod_lan::socket: Sent multicast announcement
Oct 05 20:19:14 theia ratmand[353991]: Oct 05 20:19:14.266  INFO ratman::daemon: Listening for API connections on socket 127.0.0.1:9020
```

Remember to restart Ratman after changing the configuration file at
`~/.config/ratmand/config.json` -- changes are not automatically
picked up (yet)!
