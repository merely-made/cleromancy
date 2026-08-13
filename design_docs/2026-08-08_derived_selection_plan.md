# Derived selection contract

## Decision

Add `Derived` as the third local selection mode. It chooses from the same
qualified weights as a cast, but derives the bounded sample deterministically
from public, disclosed input. It is replayable by another host without access
to local entropy.

`Derived` is not a claim of cryptographic randomness. `Cast` remains the mode
for a fresh operating-system CSPRNG choice. `Calculated` remains the mode for
the highest qualified weight.

## Inputs and receipt

`DerivedSelection` carries a nonempty public `seed` and `domain`. The receipt
stores that descriptor, the resulting bounded sample, its BLAKE3 derivation
digest, qualified weights, context and field digests, and an explicit
rejection-sampling algorithm identifier. There is no entropy nonce.

The derivation hashes length-delimited seed, domain, context digest, field
digest, qualified weights, total weight, and attempt counter. Rejection
sampling avoids modulo bias. Replay recomputes the sample and rejects any
changed descriptor, sample, nonce, or selection result.

## Product seam

The headed local consultation form exposes seed and domain only for a single
card reading. The selected values return from graph truth through the sealed
receipt. Multi-position layouts remain fresh casts. The existing Graphshell
composition intent does not gain a derivation descriptor and explicitly
rejects `Derived`; this contract is local until a separate game-facing intent
schema is designed.

## Acceptance receipts

- The same context, field, seed, and domain produce byte-equal receipts on
  independent engines and replay exactly.
- A uniform die can be derived even though it cannot be calculated.
- Tampering the seed, domain, sample, or adding an entropy nonce fails replay.
- The local consultation path persists and reopens the descriptor.

## Stop rule

This slice does not introduce a shared challenge protocol, secret material,
automatic synchronization, a game-facing derivation intent, ephemeris
calculation, correspondences, or pattern claims.
