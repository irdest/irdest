#zbus-nm

This crate has a script for generating the internal proxies. These are not meant to be user-facing. See process_proxies.py for more details.

Known issues:

 - VPNPlugin breaks for some reason.
 - zbus-xmlgen does not adequately rename properties, signals and methods if there is a naming conflict in the Rust interface. This means that NetworkManager's state functions, properties and signals are currently manually removed.
