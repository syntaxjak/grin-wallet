## Multisig wallet development progress

### Completed so far

- Added FROST conversion helpers and state storage inside `libwallet`.
- Owner API (and JSON-RPC) now exposes FROST session init, commitment/signature recording, and signing-state retrieval.
- CLI `grin-wallet multisig` commands cover:
  - Configuration (`init`, `status`, `sessions`, `approve`).
  - FROST session bootstrap (`frost-init`, `frost-session`).
  - Round message flow helpers (`frost-commitment`, `frost-signature`).
  - Signing package generation (`frost-signing-package`).
- LC/controller orchestrates finalization by detecting FROST sessions and using aggregated signatures automatically.
- Added regression test covering the full holder workflow (share export -> commitment/signature generation -> coordinator aggregation) to ensure co-signers can always contribute their portion safely.

### Next steps under consideration

- Add orchestration helpers that aggregate recorded commitments/signature shares automatically and finalize/post once the threshold is satisfied (currently manual via CLI commands).
- Extend the owner JSON-RPC with high-level endpoints for full signing orchestration (list outstanding commitments/signatures, aggregate, finalize).
- Provide convenience scripts/templates for holders (e.g. payload export/import helpers, QR support for commitments/shares).
- Expand the new regression test coverage into higher-level integration tests across the controller + CLI layers (still optional/ignored by default if needed).
- Explore automated artifact delivery (e.g. email, webhook, RPC) so holders receive per-session commitments/signing packages/signature requests without manual file passing.
- Investigate replacing the simple approval token gate with automated out-of-band signalling (email acknowledgements, push notifications, etc.) so the coordinator doesn’t have to juggle secret tokens.
- Consider whether the approval-token gate can be removed entirely in favor of “pure” FROST enforcement (threshold signatures only). Requires a review of trust/trade-off implications.
- Research decentralized coordinator variants (e.g. all holders keep their own state and any quorum subset can gather commitments/signatures without a single trusted coordinator). Document risks such as share duplication and network coordination complexity.
- Update README / docs with an end-to-end tutorial for the multisig workflow.