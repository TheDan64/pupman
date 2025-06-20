# Proxmox UnPrivileged Manager

`pupman` is a lightweight CLI tool for managing UID/GID mappings for unprivileged LXC containers. It provides a clean interface to allocate and validate ID ranges defined in `/etc/subuid` and `/etc/subgid`, and ensures consistency and isolation in your container configurations.

## 🔍 Why This Matters

Unprivileged LXC containers rely on UID/GID mapping to isolate the container's root user from the host. Mismanaged or overlapping mappings can:

- Break container startup
- Lead to privilege escalation risks
- Create hard-to-debug permission issues

`pupman` takes the guesswork out of ID management by giving you a clear view of how your subuid/subgid space is used—and helps you avoid stepping on your own feet.

<!-- TODO: Add a screencap of the app in use -->

## ✨ Feature Roadmap

- 🔍 Scan for issues:
  - [x] Validate UID/GID ranges for conflicts
  - [x] Validate user does not appear more than once in `/etc/subuid` and `/etc/subgid`
  - [x] Validate lxc.idmap values fit the container's UID/GID space
  - [x] Validate rootfs is owned by the container's UID/GID user
  - [ ] Validate lxc.idmap values do not overlap eachother
  - [ ] Validate lxc.idmap exists at all
  - [ ] Multi-node validation: no overlaps between nodes
- 🛠️ Fix scanned issues:
  - [ ] Generate valid `lxc.idmap` entries
  - [ ] Others

## 📦 Installation

### Cargo

If you already have Rust installed, you can use Cargo to add `pupman` as a dependency in your project:

```bash
cargo install pupman
```

### Curl

Coming soon!

## 🛡️ Disclaimer

This project is not affiliated with or endorsed by Canonical Ltd., the LinuxContainers project, Proxmox, or the developers of LXC.
All trademarks, including "LXC", are the property of their respective owners.
