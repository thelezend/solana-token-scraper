//! Contains the custom WebSocket message types used by the client and server.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Represents the different operation codes for WebSocket messages.
#[derive(Serialize, Deserialize, Debug)]
pub enum OpCode {
    /// Sent by the server to initiate a connection.
    Hello = 10,
    /// Sent by the client to keep the connection alive.
    Heartbeat = 1,
    /// Sent by the server to acknowledge a heartbeat.
    HeartbeatAck = 11,
    /// Sent by the client to log in.
    Login = 2,
    /// Sent by the server to disconnect the client.
    Disconnection = 3,
    /// Sent by the server when the connection is ready.
    Ready = 4,
    /// Sent by the server to monitor events.
    Monitor = 0,
}

impl OpCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            OpCode::Hello => "hello",
            OpCode::Heartbeat => "heartbeat",
            OpCode::HeartbeatAck => "heartbeat_ack",
            OpCode::Login => "login",
            OpCode::Disconnection => "disconnection",
            OpCode::Ready => "ready",
            OpCode::Monitor => "monitor",
        }
    }

    pub fn from_u64(value: u64) -> Option<Self> {
        match value {
            10 => Some(OpCode::Hello),
            1 => Some(OpCode::Heartbeat),
            11 => Some(OpCode::HeartbeatAck),
            2 => Some(OpCode::Login),
            3 => Some(OpCode::Disconnection),
            4 => Some(OpCode::Ready),
            0 => Some(OpCode::Monitor),
            _ => None,
        }
    }
}

impl FromStr for OpCode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hello" => Ok(OpCode::Hello),
            "heartbeat" => Ok(OpCode::Heartbeat),
            "heartbeat_ack" => Ok(OpCode::HeartbeatAck),
            "login" => Ok(OpCode::Login),
            "disconnection" => Ok(OpCode::Disconnection),
            "ready" => Ok(OpCode::Ready),
            "monitor" => Ok(OpCode::Monitor),
            _ => Err(()),
        }
    }
}

/// Represents the Hello event received from the server.
#[derive(Serialize, Deserialize, Debug)]
pub struct HelloEvent {
    /// The operation code for the Hello event.
    pub op: OpCode,
    /// The interval at which heartbeats should be sent.
    pub heartbeat_interval: u64,
}

/// Represents the Heartbeat event sent by the client.
#[derive(Serialize, Deserialize, Debug)]
pub struct HeartbeatEvent {
    /// The operation code for the Heartbeat event.
    pub op: OpCode,
}

/// Represents the Heartbeat ACK event received from the server.
#[derive(Serialize, Deserialize, Debug)]
pub struct HeartbeatAckEvent {
    /// The operation code for the Heartbeat ACK event.
    pub op: OpCode,
}

/// Represents the Login event sent by the client.
#[derive(Serialize, Deserialize, Debug)]
pub struct LoginEvent {
    /// The operation code for the Login event.
    pub op: OpCode,
    /// The token used for authentication.
    pub token: String,
}

/// Represents the Disconnection event received from the server.
#[derive(Serialize, Deserialize, Debug)]
pub struct DisconnectionEvent {
    /// The operation code for the Disconnection event.
    pub op: OpCode,
    /// The reason for disconnection.
    pub text: String,
}

/// Represents the Ready event received from the server.
#[derive(Serialize, Deserialize, Debug)]
pub struct ReadyEvent {
    /// The operation code for the Ready event.
    pub op: OpCode,
}

/// Represents the Monitor event received from the server.
#[derive(Serialize, Deserialize, Debug)]
pub struct MonitorEvent {
    /// The operation code for the Monitor event.
    pub op: OpCode,
    /// The data contained in the Monitor event.
    pub d: MonitorData,
}

/// Represents the data contained in a Monitor event.
#[derive(Serialize, Deserialize, Debug)]
pub struct MonitorData {
    /// The task information in the Monitor event.
    pub task: Task,
    /// The data related to the task, in JSON format.
    pub data: serde_json::Value,
}

/// Represents the task information in a Monitor event.
#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    /// The user associated with the task.
    pub user: String,
    /// Additional information about the user.
    pub user_info: String,
    /// The reason for the task.
    pub reason: Reason,
}

/// Represents the reason for a Monitor event.
#[derive(Serialize, Deserialize, Debug)]
pub enum Reason {
    /// A new tweet was detected.
    NewTweet,
    /// A new following was detected.
    NewFollowing,
    /// The user information was updated.
    UserUpdate,
}
