---
layout: page
---

<h1 class="download-header">Irdest Downloads <img src="/img/ratman.png" height="350" /></h1>

Before downloading, please beware that this is **alpha stage
software** and as such it will have bugs and produce crashes.  Please
do not expect to be able to rely on this software in your day-to-day
setup.

That being said: we want to improve Irdest for everyone so if you
experience a crash, please report the issue to our [issue
tracker][issues] or our [community mailing ist][mail]!

[issues]: https://git.irde.st/we/irdest/-/issues
[mail]: https://lists.irde.st/archives/list/community@lists.irde.st/


<!-- <img src="/img/ratman-banner.png" width="800px" /> -->

There are several ways to install Irdest.  Check the instructions for
your platform below.

<div class="tabs">
   <div class="tab">
       <input type="radio" id="tab-1" name="tab-group-1" checked>
       <label for="tab-1">Linux</label>
       <div class="content">
           <p>Currently only **Linux** systems running **systemd** are actively being tested on!</p>
       </div> 
   </div>
   <div class="tab">
       <input type="radio" id="tab-2" name="tab-group-1">
       <label for="tab-2">macOS</label>
       <div class="content">
           <p>Static (stand-alone) builds for macOS are available on both x86_64 and aarch64!</p>
       </div> 
   </div>
    <div class="tab">
       <input type="radio" id="tab-3" name="tab-group-1">
       <label for="tab-3">Other</label>
       <div class="content">
           stuff
       </div> 
   </div>
</div>




### Distribution packages

You can find pre-made packages for several Linux and BSD
distributions.  Consult the following table for details.

[![Packaging status](https://repology.org/badge/vertical-allrepos/ratman.svg)](https://repology.org/project/ratman/versions)


### Static/ portable binaries

As part of our CI pipeline we build static binaries for Ratman and
associated tools.  You can grab the latest successful build of the
Ratman release branch below

- Ratman [x86_64 binaries](https://git.irde.st/we/irdest/-/jobs/artifacts/ratman-0.4.0/raw/ratman-bundle-x86_64.tar.gz?job=bundle-ratman)!
- Ratman [aarch64 binaries](https://git.irde.st/we/irdest/-/jobs/artifacts/ratman-0.4.0/raw/ratman-bundle-aarch64.tar.gz?job=bundle-ratman-aarch64)
