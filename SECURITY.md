# Security

## Password handling

`gnome-keyring-unlock` reads the GNOME login-keyring password from standard
input and sends it to the already-running GNOME Keyring daemon through its
per-user Unix control socket.

The core Rust utility does not intentionally:

- write the password to disk;
- place the password in a command-line argument;
- log the password;
- store the password for later use.

The password necessarily exists in the process memory while the unlock request
is being performed.

## One-prompt SSH example

The optional example login controller demonstrates how the same passphrase can
be used to unlock both a GNOME login keyring and an encrypted OpenSSH private
key.

That shell example temporarily stores the passphrase in a shell variable and
exports it while invoking a private SSH askpass helper.

This has a larger exposure surface than using the core Rust utility alone.
Users should decide whether that trade-off is appropriate for their system.

## Do not store credentials in scripts

Do not hard-code passwords, private-key passphrases, access tokens or other
credentials in:

- shell scripts;
- desktop files;
- environment files;
- source code;
- command-line arguments;
- Git repositories.

## Process permissions

The utility is intended to run as the same logged-in user who owns the GNOME
Keyring daemon and its control socket.

It should not normally be run with `sudo` or as root.

## Compromised desktop sessions

This utility does not attempt to protect credentials from an attacker who has
already compromised the user's desktop session.

An unlocked keyring necessarily makes its services available to authorised
applications in that session.

## Reporting security problems

Do not include real passwords, private keys, access tokens, keyring contents or
other credentials in a public issue.

Provide only the minimum diagnostic information necessary to reproduce the
problem.
