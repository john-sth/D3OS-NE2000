<p align="center">
  <a href="https://www.uni-duesseldorf.de/home/en/home.html"><img src="media/d3os.png" width=460></a>
</p>

**A new distributed operating system for data centers, developed by the [operating systems group](https://www.cs.hhu.de/en/research-groups/operating-systems.html) of the department of computer science at [Heinrich Heine University Düsseldorf](https://www.hhu.de)**

<p align="center">
  <a href="https://www.uni-duesseldorf.de/home/en/home.html"><img src="media/hhu.svg" width=300></a>
</p>

<p align="center">
  <a href="https://github.com/hhu-bsinfo/D3OS/actions/workflows/build.yml"><img src="https://github.com/hhu-bsinfo/D3OS/actions/workflows/build.yml/badge.svg"></a>
  <img src="https://img.shields.io/badge/Rust-2024-blue.svg">
  <img src="https://img.shields.io/badge/license-GPLv3-orange.svg">
</p>

## Fork of D3OS for the Bachelor Thesis **Development of an NE2000 network card driver in Rust for an x86-based operating system** by Johann Spenrath

- this fork adds a driver for supporting NE2000-compatible Devices in D3OS
- **Structure of the new implementations:**
  - D3OS-NE2000/os/src/kernel/device/ne2k : folder contains all relevant driver code, more can be read in the included **README.md**
  - D3OS-NE2000/os/src/application/nettest: nettest application for benchmarks in user space
