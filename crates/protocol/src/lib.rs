#![forbid(unsafe_code)]

//! Versioned session contract and fail-closed authorization state machine.
//! No network or operating-system privileges are granted by this crate.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;
use uuid::Uuid;

pub const PROTOCOL_VERSION: u16 = 1;

/// Upper bound on the lifetime of an accepted remote session.
pub const MAX_ACTIVE_SESSION_MS: u64 = 8 * 60 * 60 * 1_000;

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
    Keyboard,
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

/// Mouse control is intentionally limited to the primary display in this
/// first Windows-only implementation. Coordinates are normalized to 0..=10000
/// so that input never contains absolute screen coordinates supplied by a peer.
pub const MAX_MOUSE_COORD: u16 = 10_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MouseEvent {
    Move { x: u16, y: u16 },
    Click { x: u16, y: u16, button: MouseButton },
    Scroll { x: u16, y: u16, steps: i8 },
}

impl MouseEvent {
    pub fn validate(self) -> Result<(), ProtocolError> {
        let (x, y) = match self {
            Self::Move { x, y } | Self::Click { x, y, .. } | Self::Scroll { x, y, .. } => (x, y),
        };
        if x > MAX_MOUSE_COORD || y > MAX_MOUSE_COORD {
            return Err(ProtocolError::InvalidInputEvent);
        }
        if let Self::Scroll { steps, .. } = self
            && (steps == 0 || !(-5..=5).contains(&steps))
        {
            return Err(ProtocolError::InvalidInputEvent);
        }
        Ok(())
    }
}

/// A deliberately narrow keyboard surface: one printable character or one
/// unmodified navigation/editing key per event. Modifiers and shortcuts are
/// never accepted by the remote protocol.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecialKey {
    Backspace,
    Delete,
    Enter,
    Tab,
    Escape,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum KeyboardEvent {
    Character { value: char },
    Special { key: SpecialKey },
}

impl KeyboardEvent {
    pub fn validate(self) -> Result<(), ProtocolError> {
        if let Self::Character { value } = self
            && !(value.is_alphanumeric() || value.is_ascii_punctuation() || value == ' ')
        {
            return Err(ProtocolError::InvalidInputEvent);
        }
        Ok(())
    }
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
        session_deadline_ms: u64,
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
    #[error("invalid remote mouse event")]
    InvalidInputEvent,
}

#[derive(Clone, Debug, Default)]
pub struct SessionMachine {
    state: SessionState,
    participants: Option<(DeviceId, DeviceId)>,
    // Host can pause frame delivery without terminating the grant.
    screen_delivery_enabled: bool,
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

    /// The participants remain known even after rejection or termination.
    pub fn participants(&self) -> Option<(DeviceId, DeviceId)> {
        self.participants
    }

    pub fn request(&mut self, request: SessionRequest, now_ms: u64) -> Result<(), ProtocolError> {
        if self.state != SessionState::Idle {
            return Err(ProtocolError::InvalidTransition);
        }
        request.validate(now_ms)?;
        self.participants = Some((request.host, request.operator));
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
            session_deadline_ms: now_ms.saturating_add(MAX_ACTIVE_SESSION_MS),
        };
        // Approval grants the ability to share, not permission for the client
        // to start capturing automatically. Only the host UI can publish.
        self.screen_delivery_enabled = true;
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
        let SessionState::Active {
            request, granted, ..
        } = &mut self.state
        else {
            return Err(ProtocolError::InvalidTransition);
        };
        if actor != request.host {
            return Err(ProtocolError::NotHost);
        }
        granted.remove(&permission);
        if permission == Capability::Screen {
            self.screen_delivery_enabled = false;
            self.state = SessionState::Ended;
        }
        Ok(())
    }

    pub fn terminate(&mut self, actor: DeviceId) -> Result<(), ProtocolError> {
        match &self.state {
            SessionState::AwaitingConsent(r) | SessionState::Active { request: r, .. }
                if actor == r.host || actor == r.operator =>
            {
                self.screen_delivery_enabled = false;
                self.state = SessionState::Ended;
                Ok(())
            }
            _ => Err(ProtocolError::InvalidTransition),
        }
    }

    pub fn expire(&mut self, now_ms: u64) -> bool {
        let expired = match &self.state {
            SessionState::AwaitingConsent(request) => request.expires_at_ms <= now_ms,
            SessionState::Active {
                session_deadline_ms,
                ..
            } => *session_deadline_ms <= now_ms,
            _ => false,
        };
        if expired {
            self.screen_delivery_enabled = false;
            self.state = SessionState::Ended;
        }
        expired
    }

    /// Host-side kill switch for the frame channel. Turning it off cannot
    /// be overridden by an operator, an in-flight upload, or the viewer UI.
    pub fn set_screen_delivery(
        &mut self,
        actor: DeviceId,
        enabled: bool,
        now_ms: u64,
    ) -> Result<(), ProtocolError> {
        let SessionState::Active {
            request,
            granted,
            session_deadline_ms,
        } = &self.state
        else {
            return Err(ProtocolError::InvalidTransition);
        };
        if request.host != actor {
            return Err(ProtocolError::NotHost);
        }
        if *session_deadline_ms <= now_ms || !granted.contains(&Capability::Screen) {
            return Err(ProtocolError::InvalidTransition);
        }
        self.screen_delivery_enabled = enabled;
        Ok(())
    }

    pub fn permits_screen_delivery(&self, now_ms: u64) -> bool {
        self.screen_delivery_enabled && self.permits(Capability::Screen, now_ms)
    }

    pub fn permits(&self, capability: Capability, now_ms: u64) -> bool {
        matches!(
            &self.state,
            SessionState::Active {
                granted,
                session_deadline_ms,
                ..
            } if *session_deadline_ms > now_ms && granted.contains(&capability)
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
        sm.accept(r.host, BTreeSet::from([Capability::Screen]), 200)
            .unwrap();
        assert!(sm.permits(Capability::Screen, 200));
        assert!(!sm.permits(Capability::Input, 200));
        // Invite expiry is not the expiry of an already accepted session.
        assert!(sm.permits(Capability::Screen, 2_001));
        assert!(!sm.permits(Capability::Screen, 200 + MAX_ACTIVE_SESSION_MS));
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
            sm.accept(
                r.host,
                BTreeSet::from([Capability::Screen, Capability::Terminal]),
                100
            ),
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
    fn host_can_pause_and_resume_frame_delivery_without_reapproval() {
        let r = request();
        let mut session = SessionMachine::default();
        session.request(r.clone(), 100).unwrap();
        session
            .accept(r.host, BTreeSet::from([Capability::Screen]), 200)
            .unwrap();
        assert!(session.permits_screen_delivery(201));
        assert_eq!(
            session.set_screen_delivery(r.operator, false, 202),
            Err(ProtocolError::NotHost)
        );
        assert!(session.permits_screen_delivery(202));
        session.set_screen_delivery(r.host, false, 203).unwrap();
        assert!(!session.permits_screen_delivery(203));
        assert!(session.permits(Capability::Screen, 203));
        session.set_screen_delivery(r.host, true, 204).unwrap();
        assert!(session.permits_screen_delivery(204));
        session.revoke(r.host, Capability::Screen).unwrap();
        assert!(!session.permits_screen_delivery(205));
        assert_eq!(
            session.set_screen_delivery(r.host, true, 205),
            Err(ProtocolError::InvalidTransition)
        );
    }

    #[test]
    fn mouse_events_are_bounded_and_reject_invalid_coordinates_or_scroll() {
        let good = MouseEvent::Click {
            x: MAX_MOUSE_COORD,
            y: 0,
            button: MouseButton::Left,
        };
        assert!(good.validate().is_ok());
        assert_eq!(
            MouseEvent::Move {
                x: MAX_MOUSE_COORD + 1,
                y: 2
            }
            .validate(),
            Err(ProtocolError::InvalidInputEvent)
        );
        assert_eq!(
            MouseEvent::Scroll {
                x: 1,
                y: 1,
                steps: 0
            }
            .validate(),
            Err(ProtocolError::InvalidInputEvent)
        );
        assert_eq!(
            MouseEvent::Scroll {
                x: 1,
                y: 1,
                steps: -6
            }
            .validate(),
            Err(ProtocolError::InvalidInputEvent)
        );
        let wire = serde_json::to_vec(&good).unwrap();
        assert_eq!(serde_json::from_slice::<MouseEvent>(&wire).unwrap(), good);
        assert!(serde_json::from_slice::<MouseEvent>(br#"{"type":"key","key":"super"}"#).is_err());
    }

    #[test]
    fn keyboard_events_reject_controls_and_shortcuts() {
        assert!(KeyboardEvent::Character { value: 'á' }.validate().is_ok());
        assert!(KeyboardEvent::Character { value: ' ' }.validate().is_ok());
        assert!(
            KeyboardEvent::Special {
                key: SpecialKey::Backspace
            }
            .validate()
            .is_ok()
        );
        assert_eq!(
            KeyboardEvent::Character { value: '\n' }.validate(),
            Err(ProtocolError::InvalidInputEvent)
        );
        assert_eq!(
            KeyboardEvent::Character { value: '\u{202e}' }.validate(),
            Err(ProtocolError::InvalidInputEvent)
        );
        assert!(
            serde_json::from_slice::<KeyboardEvent>(br#"{"type":"character","value":"ab"}"#)
                .is_err()
        );
        assert!(
            serde_json::from_slice::<KeyboardEvent>(br#"{"type":"special","key":"control"}"#)
                .is_err()
        );
    }

    #[test]
    fn active_session_expires_on_its_own_deadline() {
        let r = request();
        let mut sm = SessionMachine::default();
        sm.request(r.clone(), 100).unwrap();
        sm.accept(r.host, BTreeSet::from([Capability::Screen]), 200)
            .unwrap();
        assert!(!sm.expire(2_001));
        assert!(sm.expire(200 + MAX_ACTIVE_SESSION_MS));
        assert!(!sm.permits(Capability::Screen, 200 + MAX_ACTIVE_SESSION_MS));
    }

    #[test]
    fn denied_request_never_becomes_active() {
        let r = request();
        let mut sm = SessionMachine::default();
        sm.request(r.clone(), 100).unwrap();
        sm.reject(r.host).unwrap();
        assert_eq!(sm.participants(), Some((r.host, r.operator)));
        assert!(!sm.permits(Capability::Screen, 100));
        assert!(matches!(sm.state(), SessionState::Rejected));
    }

    #[test]
    fn expired_and_invalid_requests_are_denied() {
        let mut r = request();
        r.expires_at_ms = 100;
        assert_eq!(
            SessionMachine::default().request(r.clone(), 100),
            Err(ProtocolError::Expired)
        );
        r.expires_at_ms = 2_000;
        r.protocol_version = 99;
        assert_eq!(
            SessionMachine::default().request(r, 100),
            Err(ProtocolError::UnsupportedVersion)
        );
    }

    #[test]
    fn host_cannot_be_same_device_as_operator() {
        let mut r = request();
        r.host = r.operator;
        assert_eq!(
            SessionMachine::default().request(r, 0),
            Err(ProtocolError::SelfConnection)
        );
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
