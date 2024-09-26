# rmsd
USB Mass Storage Device flasher/cloner

# What is it?
*rmsd* is a fast and lightweight utility written completely in memory-safe rust, it comes with its own small userland driver designed to interact directly with USB flash drives and other devices that offer mass storage capabilities (such as external HDDs).

# Why?
This was a project i started because i was interested in how USB devices work and how Operating Systems interact with them, i also never written a driver which i consider a great experience.
While this may sound more interesting than what more traditional ISO flasher programs do, **I did not make this software with the intention of making a better product**, i also **do not perform nor provide any benchmarks/comparisons, it is not in my interest**.

# Notes for running on Linux
You will need to run the program with higher permissions, unless the user running the program has write access to the usb bus

Please not that currently **all tests were carried out on Linux**.
