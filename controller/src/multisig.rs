use crate::Error;
use grin_wallet_libwallet::{self, multisig as lib_ms, Slate};
use std::path::Path;

pub use lib_ms::{
	MultiSigConfig, MultiSigParticipant, MultiSigSession, ParticipantInitResult,
	ParticipantStorage, SessionApproval, SessionStatus,
};

pub fn status(base: &Path) -> Result<Option<MultiSigConfig>, Error> {
	lib_ms::status(base).map_err(Error::from)
}

pub fn is_configured(base: &Path) -> Result<bool, Error> {
	lib_ms::is_configured(base).map_err(Error::from)
}

pub fn initialize(
	base: &Path,
	threshold: u8,
	participants: Vec<String>,
) -> Result<Vec<ParticipantInitResult>, Error> {
	lib_ms::initialize(base, threshold, participants).map_err(Error::from)
}

pub fn create_pending_session(base: &Path, slate: &Slate) -> Result<(), Error> {
	lib_ms::create_pending_session(base, slate).map_err(Error::from)
}

pub fn ensure_threshold(base: &Path, slate: &Slate) -> Result<(), Error> {
	lib_ms::ensure_threshold(base, slate).map_err(Error::from)
}

pub fn mark_finalized(base: &Path, slate: &Slate) -> Result<(), Error> {
	lib_ms::mark_finalized(base, slate).map_err(Error::from)
}

pub fn approve(
	base: &Path,
	session_id: &str,
	participant_id: &str,
	token: &str,
) -> Result<MultiSigSession, Error> {
	lib_ms::approve(base, session_id, participant_id, token).map_err(Error::from)
}

pub fn list_sessions(base: &Path, include_finalized: bool) -> Result<Vec<MultiSigSession>, Error> {
	lib_ms::list_sessions(base, include_finalized).map_err(Error::from)
}

pub fn participant_storage_path(base: &Path, participant_id: &str) -> std::path::PathBuf {
	lib_ms::participant_storage_path(base, participant_id)
}

pub fn load_participant_storage(
	base: &Path,
	participant_id: &str,
) -> Result<Option<ParticipantStorage>, Error> {
	lib_ms::load_participant_storage(base, participant_id).map_err(Error::from)
}
