# Security policy

## Supported versions

The project is pre-alpha. Only the latest commit on `main` is supported.

## Reporting a vulnerability

Please do **not** open a public issue for a security problem. Use GitHub's private vulnerability reporting: open the repository's **Security** tab and choose **Report a vulnerability**. Include what you found, how to reproduce it, and the commit you tested.

You can expect an acknowledgement, and a fix or a decision, as soon as the maintainers are able; there is no guaranteed response time yet.

## Scope

Of particular interest: memory safety in the `unsafe` code (Vulkan, windowing, audio), and parsing of untrusted network data from a server (packet, chunk, NBT and text-component decoding) that could crash the client or corrupt memory.

Out of scope: vulnerabilities in Minecraft itself, in the Pumpkin server, or in third-party crates (report those upstream), and anything that needs a modified local `.data/` reference.
