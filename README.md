# rproc
A basic, transparent processor emulator for learning how processors and peripherals work, and easy programming.

This project aims to provide a very simple processor emulator, with a reduced instruction set, 
and easy to understand hardware interfaces. For example, besides PC, all registers are memory mapped, 
all hardware such as video is directly memory mapped, and can be written to by adressing the right location.

Aditionally, the emulator can be controlled from an ncurses based IDE, that attempts to provide insight and 
control over the processor-emulator, without a steep learning curve. It tries to provide full transparancy 
over the processor-instructions, mnemonics, subsequent opcodes, and the resulting actions in memory.

This work has been derived from the kondrak/rust64 Commodore64 emulator that was written in rust. 
Many thanks to Kondrak for providing his code.
