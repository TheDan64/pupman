# Proxmox UnPrivileged Manager

Pupman is an unofficial tool (not affiliated with Proxmox) for managing unprivileged lxc containers' gid and uid mappings in Proxmox 8.x.
The goal is to have one centralized tool for making sure there are no overlapping mappings and
being able to fix them as well.

Note that it is very much still in development and may break your build. Please backup Proxmox
and any important configs before trying it out.
