#![forbid(unsafe_code)]

//! Versioned session contract and fail-closed authorization state machine.
//! No network or operating-system privileges are granted by this crate.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;
use uuid::Uuid;

pub const PROTOCOL_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DeviceId(pub Uuid);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(pub Uuid);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Screen,
    Input,
    ClipboardRead,
    ClipboardWrite,
    FileRead,
    FileWrite,
    Chat,
    Audio,
    Restart,
    Terminal,
    TcpTunnel,
    Record,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionRequest {
    pub protocol_version: u16,
    pub session_id: SessionId,
    pub operator: DeviceId,
    pub host: DeviceId,
    pub requested: BTreeSet<Capability>,
    pub expires_at_ms: u64,
}

impl SessionRequest {
    pub fn validate(&self, now_ms: u64) -> Result<(), ProtocolError> {
        if self.protocol_version != PROTOCOL_VERSION {
            return Err(ProtocolError::UnsupportedVersion);
        }
        if self.operator == self.host {
            return Err(ProtocolError::SelfConnection);
        }
        if self.requested.is_empty() || !self.requested.contains(&Capability::Screen) {
            return Err(ProtocolError::InvalidCapabilities);
        }
        if self.expires_at_ms <= now_ms {
            return Err(ProtocolError::Expired);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionState {
    Idle,
    AwaitingConsent(SessionRequest),
    Active {
        request: SessionRequest,
        granted: BTreeSet<Capability>,
    },
    Rejected,
    Ended,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ProtocolError {
    #[error("unsupported protocol version")]
    UnsupportedVersion,
    #[error("host and operator must be different")]
    SelfConnection,
    #[error("a screen capability must be requested")]
    InvalidCapabilities,
    #[error("request expired")]
    Expired,
    #[error("transition is not allowed for current state")]
    InvalidTransition,
    #[error("only the requested host can make this decision")]
    NotHost,
    #[error("permissions cannot exceed the requested scope")]
    ScopeEscalation,
    #[error("screen access is required before control")]
    MissingScreen,
}

#[derive(Clone, Debug, Default)]
pub struct SessionMachine {
    state: SessionState,
}

impl Default for SessionState {
    fn default() -> Self {
        Self::Idle
    }
}

impl SessionMachine {
    pub fn state(&self) -> &SessionState {
        &self.state
    }

    pub fn request(&mut self, request: SessionRequest, now_ms: u64) -> Result<(), ProtocolError> {
        if self.state != SessionState::Idle {
            return Err(ProtocolError::InvalidTransition);
        }
        request.validate(now_ms)?;
        self.state = SessionState::AwaitingConsent(request);
        Ok(())
    }

    pub fn accept(
        &mut self,
        actor: DeviceId,
        granted: BTreeSet<Capability>,
        now_ms: u64,
    ) -> Result<(), ProtocolError> {
        let SessionState::AwaitingConsent(request) = &self.state else {
            return Err(ProtocolError::InvalidTransition);
        };
        request.validate(now_ms)?;
        if actor != request.host {
            return Err(ProtocolError::NotHost);
        }
        if granted.is_empty() || !granted.is_subset(&request.requested) {
            return Err(ProtocolError::ScopeEscalation);
        }
        if !granted.contains(&Capability::Screen) {
            return Err(ProtocolError::MissingScreen);
        }
        self.state = SessionState::Active {
            request: request.clone(),
            granted,
        };
        Ok(())
    }

    pub fn reject(&mut self, actor: DeviceId) -> Result<(), ProtocolError> {
        let SessionState::AwaitingConsent(request) = &self.state else {
            return Err(ProtocolError::InvalidTransition);
        };
        if actor != request.host {
            return Err(ProtocolError::NotHost);
        }
        self.state = SessionState::Rejected;
        Ok(())
    }

    pub fn revoke(&mut self, actor: DeviceId, permission: Capability) -> Result<(), ProtocolError> {
        let SessionState::Active { request, granted } = &mut self.state else {
            return Err(ProtocolError::InvalidTransition);
        };
        if actor != request.host {
            return Err(ProtocolError::NotHost);
        }
        granted.remove(&permission);
        if permission == Capability::Screen {
            self.state = SessionState::Ended;
        }
        Ok(())
    }

    pub fn terminate(&mut self, actor: DeviceId) -> Result<(), ProtocolError> {
        match &self.state {
            SessionState::AwaitingConsent(r) | SessionState::Active { request: r, .. }
                if actor == r.host || actor == r.operator =>
            {
                self.state = SessionState::Ended;
                Ok(())
            }
            _ => Err(ProtocolError::InvalidTransition),
        }
    }

    pub fn expire(&mut self, now_ms: u64) -> bool {
        let expired = match &self.state {
            SessionState::AwaitingConsent(r) | SessionState::Active { request: r, .. } => {
                r.expires_at_ms <= now_ms
            }
            _ => false,
        };
        if expired {
            self.state = SessionState::Ended;
        }
        expired
    }

    pub fn permits(&self, capability: Capability, now_ms: u64) -> bool {
        matches!(
            &self.state,
            SessionState::Active { request, granted }
                if request.expires_at_ms > now_ms && granted.contains(&capability)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> SessionRequest {
        SessionRequest {
            protocol_version: PROTOCOL_VERSION,
            session_id: SessionId(Uuid::new_v4()),
            operator: DeviceId(Uuid::new_v4()),
            host: DeviceId(Uuid::new_v4()),
            requested: BTreeSet::from([Capability::Screen, Capability::Input]),
            expires_at_ms: 2_000,
        }
    }

    #[test]
    fn accepted_scoped_session_allows_only_granted_capabilities() {
        let r = request();
        let mut sm = SessionMachine::default();
        sm.request(r.clone(), 100).unwrap();
        sm.accept(r.host, BTreeSet::from([Capability::Screen]), 200).unwrap();
        assert!(sm.permits(Capability::Screen, 200));
        assert!(!sm.permits(Capability::Input, 200));
        assert!(!sm.permits(Capability::Screen, 2_001));
    }

    #[test]
    fn rejects_unapproved_or_escalated_access() {
        let r = request();
        let mut sm = SessionMachine::default();
        assert!(!sm.permits(Capability::Screen, 100));
        sm.request(r.clone(), 100).unwrap();
        assert_eq!(
            sm.accept(r.operator, BTreeSet::from([Capability::Screen]), 100),
            Err(ProtocolError::NotHost)
        );
        assert_eq!(
            sm.accept(r.host, BTreeSet::from([Capability::Screen, Capability::Terminal]), 100),
            Err(ProtocolError::ScopeEscalation)
        );
        assert!(!sm.permits(Capability::Screen, 100));
    }

    #[test]
    fn revocation_and_termination_are_fail_closed() {
        let r = request();
        let mut sm = SessionMachine::default();
        sm.request(r.clone(), 100).unwrap();
        sm.accept(r.host, r.requested.clone(), 100).unwrap();
        sm.revoke(r.host, Capability::Input).unwrap();
        assert!(!sm.permits(Capability::Input, 101));
        sm.revoke(r.host, Capability::Screen).unwrap();
        assert_eq!(sm.state(), &SessionState::Ended);
        assert_eq!(sm.request(r, 100), Err(ProtocolError::InvalidTransition));
    }

    #[test]
    fn denied_request_never_becomes_active() {
        let r = request();
        let mut sm = SessionMachine::default();
        sm.request(r.clone(), 100).unwrap();
        sm.reject(r.host).unwrap();
        assert!(!sm.permits(Capability::Screen, 100));
        assert!(matches!(sm.state(), SessionState::Rejected));
    }

    #[test]
    fn expired_and_invalid_requests_are_denied() {
        let mut r = request();
        r.expires_at_ms = 100;
        assert_eq!(SessionMachine::default().request(r.clone(), 100), Err(ProtocolError::Expired));
        r.expires_at_ms = 2_000;
        r.protocol_version = 99;
        assert_eq!(SessionMachine::default().request(r, 100), Err(ProtocolError::UnsupportedVersion));
    }

    #[test]
    fn host_cannot_be_same_device_as_operator() {
        let mut r = request();
        r.host = r.operator;
        assert_eq!(SessionMachine::default().request(r, 0), Err(ProtocolError::SelfConnection));
    }

    #[test]
    fn session_cannot_be_accepted_after_expiry() {
        let r = request();
        let mut sm = SessionMachine::default();
        sm.request(r.clone(), 100).unwrap();
        assert_eq!(
            sm.accept(r.host, BTreeSet::from([Capability::Screen]), 2_000),
            Err(ProtocolError::Expired)
        );
        assert!(sm.expire(2_001));
    }
}
