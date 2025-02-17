# Small PoC Solana Program and Python anchorpy client

This is a simple PoC Solana Program that initializes, resizes and updates a PDA
Account.

The PDA Account's data contains a message that the client sets and gets (echos).

## Installation

### Solana

Install the Solana CLI and dependencies:

```sh
curl --proto '=https' --tlsv1.2 -sSfL https://raw.githubusercontent.com/solana-developers/solana-install/main/install.sh | bash
```

After installation you should see output similar to the following:

```
Installed Versions:Rust: rustc 1.84.1 (e71f9a9a9 2025-01-27)Solana CLI:
solana-cli 2.0.26 (src:3dccb3e7; feat:607245837, client:Agave)Anchor CLI:
anchor-cli 0.30.1Node.js: v23.7.0Yarn: 1.22.1
```

Create a solana config:

```sh
solana config set --url devnet
```

This will ensure that you are connected to the devnet and can test your program
without spending your valuable SOL.

Generate a keypair:
```sh
mkdir -p ~/.config/solana/
solana-keygen new -o ~/.config/solana/id.json
```

This will ensure that your wallet is created and can be used to sign
transactions.

Get your wallets public key:

```sh
solana-keygen pubkey ~/.config/solana/id.json
```

Get some SOL:

```
solana airdrop 5
```

If that fails, you can use the faucet at https://faucet.solana.com/ (Be sure to
select `devnet`).

## Deploy the Solana Program

Ensure you're in the root directory of the project's repository:

```
solana program deploy target/deploy/resizable_pda.so
```

## Testing the Program with anchorpy Client

From the root directory of the project, go to the `app` direcotry.

### Install dependencies

1. Ensure that you have `pyenv` installed.

2. Install Python 3.13

```sh
pyenv install 3.13
```

3. Install poetry

```sh
pip install poetry
```

4. Install the dependencies

```sh
poetry install --no-root
```

### Run the program

1. The program's help:

```sh
poetry run ./app.py --help
```

Should output the following:

```sh
Usage: app.py [OPTIONS] MESSAGE

  CLI command to create or update a PDA with a given message.

Arguments:
  MESSAGE  The message to store in the PDA.  [required]

Options:
  --nonce INTEGER                 The nonce for PDA derivation (default: 1).
                                  [default: 1]
  --install-completion [bash|zsh|fish|powershell|pwsh]
                                  Install completion for the specified shell.
  --show-completion [bash|zsh|fish|powershell|pwsh]
                                  Show completion for the specified shell, to
                                  copy it or customize the installation.
  --help                          Show this message and exit.
```

2. Run the program with the message "Hello, World!"

```sh
poetry run ./app.py "Hello Wolrd!" --nonce 1
```

~~**Note:** Be sure to use a different nonce each time you run the program (the
program's resizing function doesn't work yet).~~

**Note:** Size changes are made in increments of up to 10 kB.
