**Pukaar v0.4.1** for **Logos Basecamp 0.3.1**. Packages for Mac with Apple silicon (`darwin-arm64`).

## What's new since v0.4.0

- **README and docs:** the front page now supports a quick read and links to focused technical, security, feature and installation guides.
- **Anatomy figures:** the README includes self-contained diagrams for two-key closure and the Logos module stack.
- **Install walkthrough:** the install page now explains source and manual installation with screenshots, responsive steps and a working root deployment route.
- **Package metadata:** the package descriptions now describe the issue tracker, private chat and meetings more clearly.

## Install

Choose one way. Then fully quit Basecamp and start it again. Closing the window is not enough.

**Way 1: add a source.** In Basecamp, open Package Manager, choose Add source, and paste:

```text
https://raw.githubusercontent.com/adysingh5711/pukaar/main/public/install/logos-repo.json
```

Choose Pukaar and Install. Accept the modules it asks for.

**Way 2: by hand.** In Package Manager, choose Install Local Package for each file, in this order:

1. `chat_module-0.3.0.lgx` (from the Logos module catalog)
2. `logos-pukaar_core-module-lib.lgx`
3. `logos-pukaar_ui-module.lgx`

If Basecamp does not have `delivery_module` 0.4.0 and `storage_module` 3.0.2, install them first. Your existing data is kept.

Check your download with `shasum -a 256 -c SHA256SUMS`.

## What you need

- Logos Basecamp 0.3.1.
- A Mac with Apple silicon. Other systems are not supported yet.
- `delivery_module` 0.4.0.
- `chat_module` 0.3.0.
- `storage_module` 3.0.2. We have not tested this version with Pukaar.

## Signature check

The publisher DID is:

```text
did:jwk:eyJjcnYiOiJFZDI1NTE5Iiwia3R5IjoiT0tQIiwieCI6IlpfZkxKcnVWR3UyYnBiR0VNMlhMTElmY2FzdTFycVkycHJZM1Z0cklRR28ifQ
```

To check the packages, run these commands:

```text
lgx keyring add publisher "<DID above>" --dir ./trusted-keys
lgx verify logos-pukaar_core-module-lib.lgx --keyring-dir ./trusted-keys
lgx verify logos-pukaar_ui-module.lgx --keyring-dir ./trusted-keys
shasum -a 256 -c SHA256SUMS
```

The official module index ships `"trustedSigners": []`, so Basecamp does not enforce signatures yet. Run these checks yourself.

## Known limits

- A group join can take several minutes.
- A person who joins a group later does not see the earlier messages.
- A message sent to an old address is not sent again.
- We have not tested the chat screens live.
- In a new chat, the first message waits about 10 seconds.

Licence: MIT OR Apache-2.0.
