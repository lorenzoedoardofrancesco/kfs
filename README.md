# KFS (Kernel From Scratch)

Welcome to our **KFS** project! This project involves booting a kernel using GRUB, implementing a basic kernel library, integrating a stack with GDT for memory segmentation, developing a shell for debugging, establishing memory and interruption handling structures, and implementing system commands for kernel interaction.

This was a group project completed with [Alixmixx](https://github.com/Alixmixx). We chose to implement this project in **Rust** and used **Docker** for compilation to ensure a consistent and isolated build environment.

Working with Rust for the first time in such a complex project was a real marathon, requiring us to adapt quickly to new paradigms and practices.

## Project Description

### KFS 1: Booting and Basic Kernel

In KFS 1, we booted a kernel using GRUB and implemented a basic kernel library. The objectives included:
- Booting the kernel using GRUB.
- Writing ASM boot code to handle the multiboot header and call the kernel's main function.
- Displaying "42" on the screen.

**Bonuses:**
- Added scroll and cursor support to the I/O interface.
- Added color support to the I/O interface.
- Implemented helpers like `printf`/`printk` for easy debugging.
- Handled keyboard entries and printed them.
- Managed different screens with keyboard shortcuts.

### KFS 2: GDT and Stack

In KFS 2, we integrated a stack with the **Global Descriptor Table (GDT)** for memory segmentation. The objectives included:
- Creating and linking a GDT with entries for kernel code, data, stack, user code, data, and stack.
- Setting the GDT at address `0x00000800`.
- Developing tools to print the kernel stack in a human-readable format.

**Bonuses:**
- Implemented a minimalistic shell with commands for debugging purposes like printing the kernel stack, reboot, and halt.

### KFS 3: Memory Management

In KFS 3, we established a memory management system. The objectives included:
- Enabling memory paging in the kernel.
- Implementing a memory structure to handle paging and memory rights.
- Defining kernel and user space.
- Creating functions to allocate, free, and get the size of memory pages.
- Handling kernel panics with appropriate responses.

**Bonuses:**
- Implemented memory dumping and debugging features in the shell.

### KFS 4: Interrupt Handling

In KFS 4, we developed an interrupt handling system. The objectives included:
- Creating an **Interrupt Descriptor Table (IDT)** and registering it.
- Implementing a signal-callback system in the kernel API.
- Creating interfaces to schedule signals, clean registers, and save the stack before a panic.
- Implementing a keyboard handling system through the IDT.

**Bonuses:**
- Added syscalls handling by the IDT.
- Enhanced the keyboard handler with multi-layout support (e.g., QWERTY, AZERTY) and basic functions like `get_line`.

> For a more detailed description of each stage, please refer to the PDFs available in this repository.

## Project Challenges

This project was the **most difficult and enduring** project we've done in the advanced cursus of École 42. Each piece of information required to develop the kernel had to be meticulously searched, understood, and implemented. We spent countless nights without sleep, tirelessly working to make everything function correctly. Resources such as [OSDev](https://wiki.osdev.org/Main_Page) were invaluable in providing guidance and insights.

## Compilation

To ensure consistency and ease of compilation, we used **Docker**. The Docker environment is set up to provide all necessary dependencies and tools to build the kernel. To compile the project, use the following commands:

First, install QEMU with:

```bash
sudo apt install qemu-system-x86
```

Then, to build and run the project, use the following command:

```bash
make run
```

## Video Demonstration

Check out the video below to see the compilation and testing process of the project:

<a href="https://asciinema.org/a/667942" target="_blank"><img src="https://asciinema.org/a/667942.svg" /></a>

## Acknowledgments

Thank you for visiting our project! Feel free to clone the repository and explore the implementation. Your feedback and contributions are welcome!