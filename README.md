# LXC ID Manager

`lxcidman` is a lightweight CLI tool for managing UID/GID mappings for unprivileged LXC containers. It provides a clean interface to allocate and validate ID ranges defined in `/etc/subuid` and `/etc/subgid`, and ensures consistency and isolation in your container configurations.

## ✨ Features

- 🔍 Scan and validate UID/GID ranges for conflicts and overlaps
- 🔐 Helps enforce user namespace isolation and prevent security issues  
- 🛠️ Ideal for homelabs, Proxmox setups, and manual LXC deployments
<!--
- 🧩 **Generate** valid `lxc.idmap` entries for unprivileged containers  
- 📂 **Sync** container configs with system-wide subuid/subgid assignments
--> 

## 📦 Installation

Coming soon

## 🔍 Why This Matters

Unprivileged LXC containers rely on UID/GID mapping to isolate the container's root user from the host. Mismanaged or overlapping mappings can:

- Break container startup
- Lead to privilege escalation risks
- Create hard-to-debug permission issues

`lxcidman` takes the guesswork out of ID management by giving you a clear view of how your subuid/subgid space is used—and helps you avoid stepping on yourself.

## 🛡️ Disclaimer

This project is not affiliated with or endorsed by Canonical Ltd., the LinuxContainers project, Proxmox, or the developers of LXC.
All trademarks, including "LXC", are the property of their respective owners.
