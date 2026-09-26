# First anchor — localnet only

This is a **localnet** anchor (RISC0_DEV_MODE=1, `logos-scaffold` local sequencer at
`http://127.0.0.1:3040`). It proves the `pukaar_registry` program deploys and the
`spel anchor` / `spel inspect` round trip works end to end. It is **not** a testnet
or mainnet anchor and carries no persistence guarantee — the localnet chain state
is thrown away when the sequencer is stopped.

## Deployment

```
program_id: 23365d872303f303c7ca56cdfbca1356e38911facd08b37bdefe5e9eb9490890
```

## Checkpoint anchored

Produced by `Node::checkpoint_json()` (`modules/pukaar_core/logic/src/node.rs`) for a
throwaway `pukaar_logic` site (admin + steward + 3 residents, one `Report` event),
built with a scratch `cargo test -- --nocapture` run, not committed:

```json
{
  "heads": [
    ["84e2932765555232244c0a5670585c249e58d5e0c0d79d4dc08c623942b5fe25", 4],
    ["c4d0ee575ca2d44372bc8a9b6d71460c5fc8dcf944b50dcd2ebcef8d80d42de2", 0]
  ],
  "heads_root": "52851f39097d0e8bfe93d9bc63229865ac6dbf92b952636f1167ea304dc91667",
  "n_events": 6,
  "signer": "84e2932765555232244c0a5670585c249e58d5e0c0d79d4dc08c623942b5fe25",
  "sig": "ea898ad7bd8f68b0f187cd1472746bb2875c30c60a8b8c5394b13d90e0934fed20478c6247330326a05eefae423c47b075aabc26d29e2bc932679c18c322d90c"
}
```

`spel anchor --help` flags matched `checkpoint_json`'s `spel` string exactly
(`--site-id --heads-root --n-events --signer-pk --sig-r --sig-s --payer`); no
`node.rs` change was needed.

## Transaction

```
tx_hash: 9cca68ddbc997ac3155a4a1c60a610fac7e6d5f328f9f76445ecfde8bdefd5b6
record PDA: Fy4ehfAjFJvrZgWnPu5fGC2bLRgbbpp3at4CjL9e35Yu
```

Submitted with:

```
spel --idl idl/pukaar_registry.json --program 23365d872303f303c7ca56cdfbca1356e38911facd08b37bdefe5e9eb9490890 -- anchor \
  --site-id 65d0b878507b261c83cc3d4b6ae38224daa9bce3d5123a762623d9d23577a59a \
  --heads-root 52851f39097d0e8bfe93d9bc63229865ac6dbf92b952636f1167ea304dc91667 \
  --n-events 6 \
  --signer-pk 84e2932765555232244c0a5670585c249e58d5e0c0d79d4dc08c623942b5fe25 \
  --sig-r ea898ad7bd8f68b0f187cd1472746bb2875c30c60a8b8c5394b13d90e0934fed \
  --sig-s 20478c6247330326a05eefae423c47b075aabc26d29e2bc932679c18c322d90c \
  --payer Public/CbgR6tj5kWx5oziiFptM7jMvrQeYY3Mzaao6ciuhSr2r
...
✅ Transaction confirmed — included in a block.
```

## Independent verification (`spel inspect`)

Both `--idl` and `--type` are required (the CLI has no default type):

```
$ spel inspect Fy4ehfAjFJvrZgWnPu5fGC2bLRgbbpp3at4CjL9e35Yu --idl idl/pukaar_registry.json --type CheckpointRecord

Account: Fy4ehfAjFJvrZgWnPu5fGC2bLRgbbpp3at4CjL9e35Yu
Data:    168 bytes
Hex:     65d0b878507b261c83cc3d4b6ae38224daa9bce3d5123a762623d9d23577a59a52851f39097d0e8bfe93d9bc63229865ac6dbf92b952636f1167ea304dc91667060000000000000084e2932765555232244c0a5670585c249e58d5e0c0d79d4dc08c623942b5fe25ea898ad7bd8f68b0f187cd1472746bb2875c30c60a8b8c5394b13d90e0934fed20478c6247330326a05eefae423c47b075aabc26d29e2bc932679c18c322d90c

{
  "heads_root": "Public/6Z8AZuxBT6ThzDz6efH3qLE7WS1NE9Epce6W9J9DHV5G",
  "n_events": "6",
  "sig_r": "Public/GnY3aU9cMjPcT8e3mo7yfsJf1Tte4EHpei82NxjHuL8C",
  "sig_s": "Public/3B1KoBeDn7qBXzGLhtb2kZryLvs3YpRHS2AcepwSiGm1",
  "signer_pk": "Public/9wjCbt3LeR2pjobwwoatDypAEumrkzeHRLYbNZA9LUWp",
  "site_id": "Public/7rSmihtDQFJGMrprhhWJnffZDipjF7XFv3eeCwENLjQM"
}
```

`spel inspect`'s generic decoder prints every `[u8; 32]` field as a base58
"Public/..." account-id-looking string — that's a display artifact of the
decoder, not a semantic claim. The raw hex line is the ground truth: it
byte-for-byte matches `heads_root` (`52851f39...`) and `n_events` (`06 00
00 00 00 00 00 00` little-endian = 6) from `checkpoint_json` above, which is
what Step 7 asks to confirm.

## Not done

Step 8 (testnet) was skipped: it's optional per the plan ("better"), and doing it
needs the hosted testnet faucet, which the task instructions say to attempt only
if localnet is fully done and a faucet works without a GUI. Localnet is fully
done; the testnet faucet wasn't attempted in this pass.
