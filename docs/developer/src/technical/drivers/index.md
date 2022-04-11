# Network drivers

In Irdest a _network driver_ is called a _netmod_ (short for _network
module_).  It is responsible for linking different _instances_ of
Ratman together through some _network channel_.

The decoupling of router and network channel means that Ratman can run
on many more devices without explicit support in the Kernel for some
kind of networking.

Because interfacing with legacy networking channels comes with a lot
of logical overhead these network modules can become quite complex in
their own right.  This section in the manual aims to document the
internal structure of each network module to allow future contributors
to more easily understand and extend the code in question.
