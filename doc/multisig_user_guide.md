## Grin Wallet Multisig User Guide

This guide describes the end-to-end workflow for the threshold multisig support
recently added to `grin-wallet`. It is written for a coordinator (the wallet
instance that actually holds the coins) and a set of remote holders who provide
policy approvals plus FROST signature shares.

The flow enforces two independent safeguards before any spend can be
finalized/post:

1. **Policy approvals** – every slate is tracked inside the wallet’s
   `multisig/` state directory. Finalization will refuse to proceed until the
   configured threshold of holders has approved the spend with their private
   token.
2. **FROST signing** – spending keys are split via FROST and only aggregated
   when enough holders submit their (nonce, commitment, signature share) data.

Both gates are enforced inside the owner/foreign APIs, so a compromised UI or
CLI cannot bypass them. All sensitive holder steps can be performed entirely
offline.

---

### 1. One-time setup

Run these steps once on the wallet that owns the outputs. Each holder only needs
the artifacts that are generated for them.

1. **Initialize the configuration**

   ```bash
   grin-wallet multisig init \
     --threshold 2 \
     --holder alice --holder bob --holder carol
   ```

   This writes `grin-wallet-data/multisig/config.json` plus one
   `multisig/participants/<holder>.json` file per label. Each file contains:

   - A random approval token
   - Metadata binding the holder label to this policy

2. **Distribute holder artifacts**

   Securely copy every holder’s `participants/<holder>.json` to the respective
   person. They must keep the contained `token` secret. The coordinator keeps a
   copy in the wallet directory so approvals can be validated.

3. **Verify the policy** (optional)

   ```bash
   grin-wallet multisig status
   ```

   Shows threshold, registered holders and where their storage files live.

---

### 2. Initiating a transaction

Any time the coordinator initiates or processes a slate (standard send or
invoice), the wallet automatically records a pending multisig session:

1. Use the regular `send`, `process-invoice`, or `finalize` commands to build a
   slate. As soon as the slate exists, the wallet writes
   `multisig/sessions/<slate_id>.json` with the amount, required approvals, and
   participating holders.

2. Share the pending slate with the counterparty as usual (Slatepack file or
   message). Once they return their part, continue with the multisig flow below
   before finalizing/posting.

3. View open sessions at any time:

   ```bash
   grin-wallet multisig sessions
   ```

---

### 3. Holder approvals (policy gate)

Each holder must acknowledge the spend by submitting the token stored in their
participant file.

1. The coordinator sends the slate ID (UUID) and amount to every holder.

2. Each holder runs:

   ```bash
   grin-wallet multisig approve \
     --session <slate_id> \
     --holder <their_label> \
     --token <token_from_json>
   ```

   Approvals can be done offline: the command only needs the token and slate
   identifier. The wallet records the approval timestamp; `multisig sessions`
   shows who has approved and who is still pending.

3. Finalization (`grin-wallet finalize`) now refuses to continue until the
   threshold count is reached. This check is executed by both owner and foreign
   APIs, so any code path that attempts to finalize automatically goes through
   the same guard.

---

### 4. FROST signing (cryptographic gate)

After policy approvals are collected, the signing key still cannot be used until
enough holders contribute fresh FROST commitments and signature shares. The flow
looks like this:

#### 4.1 Coordinator bootstraps the FROST session

```bash
# Provide either --input <file> or --slatepack <inline message>
grin-wallet multisig frost-init \
  --input pending_slatepack.txt \
  --session-out session_summary.json
```

- Persists the FROST session for this slate inside the wallet context.
- Produces `session_summary.json` describing participants, threshold and current
  signing-state snapshot.

#### 4.2 Export per-holder share bundle

Repeat for each holder:

```bash
grin-wallet multisig frost-export-share \
  --slate-id <slate_id> \
  --label <holder_label> \
  --outfile <holder>_share.json
```

Securely deliver the JSON file to the holder. It contains the serialized FROST
identifier/key package for that slate only (enough to produce commitments &
signatures but not to reconstruct the full key).

#### 4.3 Holder: generate commitment & nonce

Each holder performs:

```bash
grin-wallet multisig frost-generate-commitment \
  --share-path <holder>_share.json \
  --nonce-out <holder>_nonces.json \
  --commitment-out <holder>_commitment.json
```

- `commitment_out` is sent back to the coordinator.
- `nonce_out` stays private (required later for the signature share).

#### 4.4 Coordinator records commitments

```bash
grin-wallet multisig frost-record-commitment \
  --slate-id <slate_id> \
  --label <holder_label> \
  --data-path <holder>_commitment.json
```

Repeat until at least the threshold number of commitments have been recorded.

#### 4.5 Coordinator builds the signing package

```bash
grin-wallet multisig frost-signing-package \
  --input pending_slatepack.txt \
  --outfile signing_package.json
```

This verifies that enough commitments exist and emits a JSON file containing the
aggregated message, threshold, and each participant’s serialized commitment.
Distribute this file to every holder that committed in step 4.4.

#### 4.6 Holder: generate signature share

```bash
grin-wallet multisig frost-generate-signature \
  --share-path <holder>_share.json \
  --nonce-path <holder>_nonces.json \
  --signing-package-path signing_package.json \
  --outfile <holder>_signature.json
```

Return the resulting signature share to the coordinator.

#### 4.7 Coordinator records signature shares

```bash
grin-wallet multisig frost-record-signature \
  --slate-id <slate_id> \
  --label <holder_label> \
  --data-path <holder>_signature.json
```

Once `threshold` shares are recorded, the wallet will aggregate them the next
time `grin-wallet finalize` runs. The internal `aggregate_signature` check
verifies that the reconstructed key matches the slate excess; if anything was
tampered with, finalization aborts.

---

### 5. Finalize and post

With both approval and FROST requirements satisfied, run the usual finalize
step:

```bash
grin-wallet finalize --input pending_slatepack.txt
```

- The owner API automatically enforces `ensure_threshold` immediately before the
  signature aggregation.
- The aggregated Schnorr signature replaces the standard single-party signature.
- Posting (`grin-wallet finalize --fluff` or `grin-wallet post`) works as usual
  once the slate is finalized.

---

### Operational tips

- **Audit sessions regularly** – `grin-wallet multisig sessions --all` provides
  a full ledger of pending and finalized slates, including who approved and when.
- **Backup the `multisig/` directory** – it is the authoritative record tying
  slate IDs to approvals and signatures. Losing it means approvals would need to
  be re-collected.
- **Offline-friendly holder tooling** – all holder commands operate on JSON
  files, so they can run on air-gapped machines. Only the coordinator wallet
  ever needs chain access.
- **Tamper resistance** – because both the policy approval check and the FROST
  aggregation happen down inside `libwallet`, custom front-ends cannot bypass
  them without modifying the wallet binary itself.

With this workflow, no transaction can be finalized unless the threshold number
of holders both approves the slate *and* contributes their unique FROST partial
signature, giving you defense-in-depth against key theft, malware, or operator
error.