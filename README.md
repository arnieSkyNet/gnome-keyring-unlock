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

## Download

For most users, the easiest installation method is to download a
pre-built binary from the GitHub Releases page:

https://github.com/arnieSkyNet/gnome-keyring-unlock/releases

Download the Linux binary appropriate for your system.

Create a personal executable directory if necessary:

    mkdir -p "$HOME/sbin"

Install the downloaded binary:

    install -m 700 ./gnome-keyring-unlock "$HOME/sbin/gnome-keyring-unlock"

The release download avoids the need to install Rust or Cargo.

If there is not yet a suitable pre-built binary for your system, use
the **Build** instructions below.

## Build

To build from source, first clone the GitHub repository:

    git clone https://github.com/arnieSkyNet/gnome-keyring-unlock.git
    cd gnome-keyring-unlock

Build the optimised release executable:

    cargo build --release

Run the automated protocol tests:

    cargo test

The resulting native executable is:

    target/release/gnome-keyring-unlock

Continue with the **Install** section below to install it for your user account.

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


### Optional SSH integration

An SSH agent is not required to unlock GNOME Keyring.

The supplied one-prompt controller always attempts to unlock the GNOME login
keyring. It will additionally load the configured SSH private key only when:

- an SSH agent is available through `SSH_AUTH_SOCK`; and
- the configured SSH private-key file exists.

This project does not start or manage an SSH agent.

To check whether an SSH agent is available in your graphical session:

    if [ -n "${SSH_AUTH_SOCK-}" ]; then
        echo "SSH agent available: $SSH_AUTH_SOCK"
    else
        echo "No SSH agent is available - GNOME Keyring unlocking can still be used"
    fi

If an agent is available, you can see which identities it currently contains:

    ssh-add -l

If you do not use SSH keys, no SSH configuration is required.

### Install the one-prompt launcher

The repository contains the controller and its private SSH askpass helper.

Install them with:

    mkdir -p "$HOME/sbin"

    install -m 700 \
        examples/one-prompt-login \
        "$HOME/sbin/gnome-keyring-one-prompt-login"

    install -m 700 \
        examples/gnome-keyring-unlock-ssh-passphrase \
        "$HOME/sbin/gnome-keyring-unlock-ssh-passphrase"

The controller expects the main Rust utility at:

    $HOME/sbin/gnome-keyring-unlock

The supplied controller uses this SSH private key:

    $HOME/.ssh/id_rsa

If your encrypted SSH private key has another name, edit the installed
controller and change its `KEY=` line. For example, an Ed25519 key might use:

    KEY="$HOME/.ssh/id_ed25519"

Never put the SSH passphrase itself in the script.

### Graphical askpass program

The example controller uses:

    /usr/libexec/ssh-askpass/x11-ssh-askpass

Check whether that program exists on your system:

    test -x /usr/libexec/ssh-askpass/x11-ssh-askpass \
        && echo "x11-ssh-askpass found" \
        || echo "x11-ssh-askpass not found at this path"

Askpass implementations and their locations vary between Linux
distributions. If yours is elsewhere, change the `ASKPASS=` line in the
installed controller.


### Test the launcher before enabling autostart

Do not reboot immediately after installing the one-prompt setup.

If you are using the optional SSH integration, you can check the agent first:

    test -n "${SSH_AUTH_SOCK-}" \
        && echo "SSH agent available" \
        || echo "No SSH agent available - SSH loading will be skipped"

Then run the controller manually:

    "$HOME/sbin/gnome-keyring-one-prompt-login"

You should receive one graphical passphrase prompt.

After entering the passphrase, use the verification commands in the next
section to confirm that the GNOME login keyring is unlocked.

If you are using the optional SSH integration, also run:

    ssh-add -l

Your configured SSH key should be listed.

Only enable automatic startup after this manual test works.

### Start automatically after graphical login

An example desktop autostart file is supplied as:

    examples/gnome-keyring-one-prompt-login.desktop

Create the autostart directory if necessary and install the file:

    mkdir -p "$HOME/.config/autostart"

    install -m 600 \
        examples/gnome-keyring-one-prompt-login.desktop \
        "$HOME/.config/autostart/gnome-keyring-one-prompt-login.desktop"

On the next graphical login, the desktop will start the controller, which
will display the single graphical passphrase prompt.

The desktop file contains no password or passphrase.

### Disable the automatic launcher

To disable the launcher without deleting it:

    mv \
        "$HOME/.config/autostart/gnome-keyring-one-prompt-login.desktop" \
        "$HOME/.config/autostart/gnome-keyring-one-prompt-login.desktop.disabled"

To enable it again:

    mv \
        "$HOME/.config/autostart/gnome-keyring-one-prompt-login.desktop.disabled" \
        "$HOME/.config/autostart/gnome-keyring-one-prompt-login.desktop"

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
