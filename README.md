# rmsd
USB Mass Storage Device flasher/cloner for all *NIX Operating Systems.

# What is it?
*rmsd* is a fast and lightweight utility written completely in memory-safe rust, it comes with its own small userland driver designed to interact directly with USB flash drives and other devices that offer mass storage capabilities (such as external HDDs).

# Why?
This was a project i started because i was interested in how USB devices work and how Operating Systems interact with them, i also have never written a driver which i consider a great experience.
While this may sound more interesting than what more traditional ISO flasher programs do, **I did not make this software with the intention of making a better product**, i also **do not perform nor provide any benchmarks/comparisons, it is not in my interest**.

# General Notes
(Please note that currently **all tests were carried out on Linux**)

- Building on Windows is possible, but usage is not what i'd consider practical, installing WinUSB for all Mass Storage Devices is required.

- You will need to run the program with higher permissions (``sudo``), unless the user running the program has write access to the usb bus.

- All USB devices that can be used as a disk (i.e. are Mass Storage Class USB devices) should be supported, as they all communicate the same way, as such, the driver implements only this common protocol (Bulk Only, also referred as "BBB"), as noted by many documents that describe this protocol, the only type of devices that do not use it are USB Floppy Disk readers and thus will not be detected.

- Some Mass Storage Devices may expect a specific sector (or block) size, this program however assumes a sector size of 512 bytes (which is common in USB flash drives), a simple workaround for this would be to set the buffer size to a multiple of the expected size, for example, if a USB CD Burner expects the common sector size for CDs (which means it expect the data size to be a multiple 2048 bytes), you would set the buffer size to a multiple of 4, because the program transfers *buffers* and not single sectors at a time for speed.
 
- As of now, the I/O commands with the lowest maximum data size are being used for maximum compatibility, namely READ(8) and WRITE(8), which impose some constraints:
    - A maximum of 65535 sectors (about 32GB) can be transferred in a single transfer, this translates to the maximum buffer size.
    - The number of addressable sectors is 2^32, this means that the maximum image size while flashing and cloning is 2TB.
