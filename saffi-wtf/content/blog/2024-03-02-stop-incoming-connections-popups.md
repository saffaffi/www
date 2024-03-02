---
---

# Stop "Allow Incoming Connections?" Popups When Developing on macOS

These days, I'm back to developing almost entirely on macOS. When I was working
on the implementation of this site, it was starting to get quite grating that
every time I ran `cargo run`, macOS would pop up a window that says "Allow
Incoming Connections?". I had to click "Allow" every single time I changed the
code and re-ran it.

In general, I know why this happens: the firewall is enabled, and Cargo doesn't
sign executables, so every time I produced a new executable, it was _nEw AnD
sCaRy_. Adding the executable to the list of trusted apps doesn't help for
exactly that reason (in fact, that's all that clicking "Allow" does). The
problem is, I _want_ the firewall to be enabled.

I tried a few things to stop this, including adding my terminal emulator as a
developer tool in the Security & Privacy preference pane, which allows it to
"run software locally that does not meet the system's security policy". That
doesn't extend to running executables that need to accept incoming connections,
apparently (at least for WezTerm).

It turns out that in my particular case, though, there's an easy solution: I had
totally overlooked that my program was binding to `0.0.0.0:<port>`. For local
development, that's unnecessary. I changed it to `127.0.0.1:<port>`, and the
popups blissfully disappeared.
