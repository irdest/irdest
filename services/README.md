# Irdest services

**If you are an end-user, looking for applications, check out
[clients](../clients) instead!**

Following is a collection of services that use irdest to do
networking.  None of them rely on centralised servers or
infrastructure, and can operate on a completely ad-hoc network.  Some
smaller example services are included as examples on how to use the
irdest-sdk in your applications, and what kind of problems it can
solve.

Note that this is potentially only a small selection of services that
use irdest.  We encourage you to write your own that don't have to be
part of this repository; we are not the single source of truth.


| Service            | Part of Irdest client bundle | Description                                                                      |
|--------------------|------------------------------|----------------------------------------------------------------------------------|
| [org.irdest.chat]  | Yes                          | Encrypted chat application, supporting DMs and group chats                       |
| [org.irdest.feed]  | Yes                          | Twitter-like micro-blogging feed                                                 |
| [org.irdest.files] | Yes                          | Filesharing utilitiy.  Both useful on it's own, and to be used by other services |
| [org.irdest.ping]  |                              | A simple ping program for a decentralised networking backend                     |
| [org.irdest.voice] | Yes                          | Encrypted voice call services, supporting single and group calls                 |
| [org.irdest.webui] | *Some*                       | A cross-platform web UI to run in the browser                                    |


[org.irdest.chat]: ./chat
[org.irdest.feed]: ./feed
[org.irdest.files]: ./files
[org.irdest.ping]: ./ping
[org.irdest.voice]: ./voice
[org.irdest.webui]: ./webui
