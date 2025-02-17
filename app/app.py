#!/usr/bin/env python3
import asyncio
import json
import os
from pathlib import Path

import typer
from anchorpy.error import \
    AccountDoesNotExistError  # Used for checking if PDA exists
from anchorpy.program.context import Context
from anchorpy.program.core import Program
from anchorpy.provider import Provider, Wallet
from anchorpy.workspace import close_workspace, create_workspace
from solana.constants import SYSTEM_PROGRAM_ID
from solana.rpc.async_api import AsyncClient
from solders.keypair import Keypair
from solders.pubkey import Pubkey

PROJECT_PATH: str = os.path.realpath(
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
)
SEED_PREFIX: bytes = b"my-seed"
DEVNET_URL: str = "https://api.devnet.solana.com"

app: typer.Typer = typer.Typer()


def load_user_keypair() -> Keypair:
    with open(f"{str(Path.home())}/.config/solana/id.json", "r") as key_file:
        key_data = json.load(key_file)
        return Keypair.from_bytes(bytes(key_data))


# Load or generate a keypair for the user
user: Keypair = load_user_keypair()


async def derive_pda(
    program: Program, nonce: int
) -> Pubkey:
    """Derives the PDA address using the seed, user pubkey, and nonce."""
    pda, _ = Pubkey.find_program_address(
        [SEED_PREFIX, bytes(program.provider.wallet.public_key), nonce.to_bytes(8, "little")],
        program.program_id,
    )
    return pda


async def create_pda(program: Program, nonce: int, message: str) -> None:
    """Creates a PDA account and stores the initial message."""
    pda: Pubkey = await derive_pda(program, nonce)

    tx = await program.rpc["create_account"](
        nonce,
        message,
        ctx=Context(
            accounts={
                "pda_account": pda,
                "user": program.provider.wallet.public_key,
                "system_program": SYSTEM_PROGRAM_ID,  # System Program ID
            },
            signers=[user],
        ),
    )

    print(f"PDA Created: {pda} | Transaction: {tx}")

    # Retrieve and print the stored message
    _ = await get_pda_message(program, pda)


async def update_pda(program: Program, nonce: int, new_message: str) -> None:
    """Resizes the PDA (if necessary) and updates the stored message."""
    pda: Pubkey = await derive_pda(program, nonce)

    # Fetch the current PDA account data
    try:
        account = await program.account["PDAAccount"].fetch(pda)
        current_size: int = len(account.data)
    except AccountDoesNotExistError:
        print(f"Error: PDA {pda} does not exist. Cannot update.")
        return

    new_size: int = len(new_message)

    # Resize the PDA only if the new message is larger than the current storage
    if new_size > current_size:
        print(f"Resizing PDA from {current_size} to {new_size} bytes...")
        tx_resize = await program.rpc["resize_account"](
            new_size,
            ctx=Context(
                accounts={
                    "pda_account": pda,
                    "authority": program.provider.wallet.public_key,
                },
                signers=[user],
            ),
        )
        print(f"PDA Resized | Transaction: {tx_resize}")

        # Wait for confirmation before updating
        await asyncio.sleep(5)

    # Now, update the stored message
    tx_update = await program.rpc["update_data"](
        new_message,
        ctx=Context(
            accounts={
                "pda_account": pda,
                "authority": program.provider.wallet.public_key,
            },
            signers=[user],
        ),
    )

    print(
        f"PDA Updated with message: {new_message} | Transaction: {tx_update}"
    )

    # Retrieve and print the updated message
    _ = await get_pda_message(program, pda)


async def get_pda_message(program: Program, pda: Pubkey) -> str | None:
    """Fetches the stored message from the PDA."""
    try:
        account = await program.account["PDAAccount"].fetch(pda)
        stored_message = (
            bytes(account.data).decode("utf-8").rstrip("\x00")
        )  # Decode and remove padding
        print(f"Retrieved PDA Message: {stored_message}")
        return stored_message
    except AccountDoesNotExistError:
        print(f"No PDA found at {pda}, needs creation.")
        return None


async def check_and_create_or_update_pda(
    program: Program, nonce: int, message: str
) -> None:
    """Checks if PDA exists. If not, creates it; otherwise, updates it."""
    pda: Pubkey = await derive_pda(program, nonce)

    stored_message: str | None = await get_pda_message(program, pda)

    if stored_message is None:
        # PDA does not exist, create it
        await create_pda(program, nonce, message)
    else:
        # PDA exists, update it
        await update_pda(program, nonce, message)


async def async_main(message: str, nonce: int):
    """Main function to handle Solana interactions."""
    # Load the Anchor program
    workspace = create_workspace(PROJECT_PATH, url=DEVNET_URL)
    program: Program = workspace["resizable_pda"]

    # Check if PDA exists; create or update accordingly
    await check_and_create_or_update_pda(program, nonce, message)

    # Close the client connection
    await close_workspace(workspace)


@app.command()
def main(
    message: str = typer.Argument(
        ..., help="The message to store in the PDA."
    ),
    nonce: int = typer.Option(
        1, help="The nonce for PDA derivation (default: 1)."
    ),
) -> None:
    """CLI command to create or update a PDA with a given message."""
    asyncio.run(async_main(message, nonce))


if __name__ == "__main__":
    app()
