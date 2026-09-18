# GNOME Keyring Unlock

A small native Rust utility for unlocking an existing GNOME login keyring by
communicating directly with the running `gnome-keyring-daemon` control socket.

It was created for Linux desktop sessions that use automatic login but still
need a password-protected GNOME login keyring.

## Why this exists

With a conventional password login, PAM can pass the user's login password to
`pam_gnome_keyring`, allowing the login keyring to be unlocked automatically.

With automatic login there is no password for PAM to pass to GNOME Keyring.
The desktop can therefore start normally while the login keyring remains
locked.

That can affect applications which expect credentials from Secret Service,
including GNOME Online Accounts.

`gnome-keyring-unlock` provides a small native building block for systems where
the user deliberately wants to enter the keyring password after automatic
login.

## What it does

The program:

1. Reads the keyring password from standard input.
2. Locates the GNOME Keyring control socket using `GNOME_KEYRING_CONTROL`, or
   falls back to `$XDG_RUNTIME_DIR/keyring/control`.
3. Connects to the already-running per-user GNOME Keyring daemon.
4. Sends the GNOME Keyring unlock control request.
5. Checks the daemon's response.
6. Exits successfully only when the daemon reports success.

It does **not** start another GNOME Keyring daemon and does not invoke
`gnome-keyring-daemon --unlock`.

The utility has no third-party Rust dependencies.

## Requirements

- Linux
- GNOME Keyring
- An already-running `gnome-keyring-daemon`
- Rust toolchain when building from source

## Build

Clone the repository and build the release executable:

    cargo build --release

The resulting native executable is:

    target/release/gnome-keyring-unlock

## Install

For a per-user installation:

    mkdir -p "$HOME/sbin"
    install -m 700 \
        target/release/gnome-keyring-unlock \
        "$HOME/sbin/gnome-keyring-unlock"

Adjust the destination if `$HOME/sbin` is not appropriate on your system.

## Basic use

The utility reads the password from standard input.

Do not put the password directly in a command-line argument, script, desktop
file or Git repository.

It is intended to be called by a trusted interactive helper which obtains the
password from the user and supplies it to `gnome-keyring-unlock` through
standard input.

## Automatic login and a single passphrase prompt

The original reason for developing this utility was a desktop using automatic
login.

Automatic login allowed the desktop session to start without entering the
normal account password, but that also meant PAM had no password available to
unlock the protected GNOME login keyring.

The machine also used an encrypted OpenSSH private key which required a
passphrase after login.

In the original installation, the SSH private key and GNOME login keyring were
deliberately protected with the same passphrase. A small login controller could
therefore:

1. Display one graphical passphrase prompt.
2. Pass that value to `gnome-keyring-unlock` through standard input.
3. Pass the same value to `ssh-add` through an SSH askpass helper.
4. Discard the shell variables holding the passphrase.
5. Restart GNOME Online Accounts so that credentials which were requested
   before the keyring became available could be retried.

Example integration scripts are provided in the `examples` directory.

The examples are not required by the Rust utility itself. Desktop startup,
SSH agents, askpass implementations and online-account services vary between
Linux installations.

## Verify that the login keyring is unlocked

On a system using Secret Service, the state of the login collection can be
checked with:

    busctl --user get-property \
        org.freedesktop.secrets \
        /org/freedesktop/secrets/collection/login \
        org.freedesktop.Secret.Collection \
        Locked

An unlocked login collection reports:

    b false

A locked login collection reports:

    b true

## Troubleshooting

### No such file or directory

Check that the normal GNOME Keyring daemon is running:

    ps -u "$USER" -o pid,ppid,args | grep '[g]nome-keyring-daemon'

Then check the runtime control directory:

    ls -la "$XDG_RUNTIME_DIR/keyring"

A normal running installation should have a `control` Unix socket in that
directory.

### Avoid starting a second keyring daemon

A separate invocation of:

    gnome-keyring-daemon --unlock

may result in an additional daemon process rather than unlocking the Secret
Service collection belonging to the existing desktop session.

`gnome-keyring-unlock` instead communicates with the control socket belonging
to the already-running daemon.

### GNOME Online Accounts still requests attention

Applications may try to obtain credentials before the login keyring has been
unlocked.

If GNOME Online Accounts did this during startup, it may need to retry after
the keyring becomes available.

The example login controller demonstrates the approach used on the original
system. Whether restarting GOA is necessary or desirable should be determined
for each desktop environment.

## Tested environment

The original implementation was developed and tested on:

- Debian 13 (Trixie), x86-64
- GNOME Keyring 48
- Xfce desktop
- GDM automatic login
- OpenSSH agent and an encrypted RSA private key
- GNOME Online Accounts

A clean reboot test confirmed that one graphical passphrase entry resulted in:

- the GNOME login keyring reporting `b false`;
- the encrypted SSH private key being loaded into the SSH agent;
- all configured GNOME Online Accounts reporting that no attention was needed;
- only the normal desktop `gnome-keyring-daemon` remaining active.

Other distributions, desktop environments and GNOME Keyring versions may
behave differently.

## Security

The core Rust utility receives the keyring password through standard input.
It does not intentionally write the password to disk or place it in a
command-line argument.

The optional one-prompt shell integration necessarily handles the passphrase
temporarily in the user's session. See `SECURITY.md` for details.

Never commit passwords, private keys, access tokens, keyring files or other
credentials to this repository.

## Licence

MIT. See `LICENSE`.
