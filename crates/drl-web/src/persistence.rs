//! Versioned fixed-session save tokens for the browser boundary.

use drl_protocol::{Command, Direction, EquipmentSlot, ItemId, Position};

const SNAPSHOT_PREFIX: &str = "DRL-RUST-BROWSER-SAVE/";
const SNAPSHOT_VERSION: &str = "1";
const SNAPSHOT_CONTENT: &str = "fixed-m4-v1";
const SNAPSHOT_MAX_BYTES: usize = 16 * 1024;
const SNAPSHOT_MAX_COMMANDS: usize = 4096;
#[cfg(any(target_arch = "wasm32", test))]
const QUARANTINE_PREFIX: &str = "DRL-RUST-BROWSER-REJECTED/1:";

/// Errors returned when a browser-session snapshot cannot be decoded or replayed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotError {
  /// The token uses a snapshot version this build does not understand.
  UnsupportedVersion(String),
  /// The token targets a different fixed-session content profile.
  UnsupportedContent(String),
  /// The token has an invalid prefix, command, number, or delimiter.
  Malformed,
  /// The token exceeds the bounded browser-session save policy.
  TooLarge,
  /// A decoded command is no longer legal for the fixed session.
  CommandRejected(String),
  /// The fixed session could not be initialized for replay.
  Initialization(String),
}

impl std::fmt::Display for SnapshotError {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::UnsupportedVersion(version) => {
        write!(formatter, "unsupported snapshot version {version}")
      }
      Self::UnsupportedContent(content) => {
        write!(formatter, "unsupported snapshot content {content}")
      }
      Self::Malformed => write!(formatter, "malformed browser-session snapshot"),
      Self::TooLarge => write!(formatter, "browser-session snapshot is too large"),
      Self::CommandRejected(error) => write!(formatter, "snapshot command rejected: {error}"),
      Self::Initialization(error) => write!(formatter, "snapshot initialization failed: {error}"),
    }
  }
}

impl std::error::Error for SnapshotError {}

fn direction_code(direction: Direction) -> Option<char> {
  Some(match direction {
    Direction::None => '0',
    Direction::North => 'n',
    Direction::NorthEast => 'e',
    Direction::East => 'r',
    Direction::SouthEast => 'd',
    Direction::South => 's',
    Direction::SouthWest => 'q',
    Direction::West => 'l',
    Direction::NorthWest => 'z',
  })
}

fn parse_direction(code: &str) -> Result<Direction, SnapshotError> {
  match code {
    "0" => Ok(Direction::None),
    "n" => Ok(Direction::North),
    "e" => Ok(Direction::NorthEast),
    "r" => Ok(Direction::East),
    "d" => Ok(Direction::SouthEast),
    "s" => Ok(Direction::South),
    "q" => Ok(Direction::SouthWest),
    "l" => Ok(Direction::West),
    "z" => Ok(Direction::NorthWest),
    _ => Err(SnapshotError::Malformed),
  }
}

fn parse_item_id(value: &str) -> Result<ItemId, SnapshotError> {
  let id = value.parse::<u64>().map_err(|_| SnapshotError::Malformed)?;
  (id > 0)
    .then_some(ItemId::new(id))
    .ok_or(SnapshotError::Malformed)
}

fn parse_position(value: &str) -> Result<Position, SnapshotError> {
  let mut parts = value.split(',');
  let x = parts
    .next()
    .ok_or(SnapshotError::Malformed)?
    .parse::<i32>()
    .map_err(|_| SnapshotError::Malformed)?;
  let y = parts
    .next()
    .ok_or(SnapshotError::Malformed)?
    .parse::<i32>()
    .map_err(|_| SnapshotError::Malformed)?;
  if parts.next().is_some() {
    return Err(SnapshotError::Malformed);
  }
  Ok(Position::new(x, y))
}

fn encode_command(command: Command) -> Result<String, SnapshotError> {
  Ok(match command {
    Command::Move(direction) => format!(
      "m{}",
      direction_code(direction).ok_or(SnapshotError::Malformed)?
    ),
    Command::AttackMelee(direction) => format!(
      "a{}",
      direction_code(direction).ok_or(SnapshotError::Malformed)?
    ),
    Command::AttackRanged(position) => format!("r{},{}", position.x, position.y),
    Command::Wait => "w".to_string(),
    Command::Pickup => "p".to_string(),
    Command::Drop(id) => format!("d{}", id.as_u64()),
    Command::Equip(id) => format!("e{}", id.as_u64()),
    Command::Unequip(slot) => format!(
      "u{}",
      if slot == EquipmentSlot::Weapon {
        "w"
      } else {
        "a"
      }
    ),
    Command::Use(id) => format!("c{}", id.as_u64()),
    Command::Reload => "l".to_string(),
    Command::Descend => "x".to_string(),
  })
}

fn decode_command(token: &str) -> Result<Command, SnapshotError> {
  if token.is_empty() {
    return Err(SnapshotError::Malformed);
  }
  let (opcode, rest) = token.split_at(1);
  match opcode {
    "m" => Ok(Command::Move(parse_direction(rest)?)),
    "a" => Ok(Command::AttackMelee(parse_direction(rest)?)),
    "r" => Ok(Command::AttackRanged(parse_position(rest)?)),
    "w" if rest.is_empty() => Ok(Command::Wait),
    "p" if rest.is_empty() => Ok(Command::Pickup),
    "d" => Ok(Command::Drop(parse_item_id(rest)?)),
    "e" => Ok(Command::Equip(parse_item_id(rest)?)),
    "u" if rest == "w" => Ok(Command::Unequip(EquipmentSlot::Weapon)),
    "u" if rest == "a" => Ok(Command::Unequip(EquipmentSlot::Armor)),
    "c" => Ok(Command::Use(parse_item_id(rest)?)),
    "l" if rest.is_empty() => Ok(Command::Reload),
    "x" if rest.is_empty() => Ok(Command::Descend),
    _ => Err(SnapshotError::Malformed),
  }
}

pub(crate) fn encode_snapshot(commands: &[Command]) -> Result<String, SnapshotError> {
  if commands.len() > SNAPSHOT_MAX_COMMANDS {
    return Err(SnapshotError::TooLarge);
  }
  let mut token = format!("{SNAPSHOT_PREFIX}{SNAPSHOT_VERSION}:{SNAPSHOT_CONTENT}:");
  for (index, command) in commands.iter().copied().enumerate() {
    if index > 0 {
      token.push(';');
    }
    token.push_str(&encode_command(command)?);
  }
  if token.len() > SNAPSHOT_MAX_BYTES {
    return Err(SnapshotError::TooLarge);
  }
  Ok(token)
}

pub(crate) fn decode_snapshot(token: &str) -> Result<Vec<Command>, SnapshotError> {
  if token.len() > SNAPSHOT_MAX_BYTES {
    return Err(SnapshotError::TooLarge);
  }
  let Some(versioned) = token.strip_prefix(SNAPSHOT_PREFIX) else {
    return Err(SnapshotError::Malformed);
  };
  let mut parts = versioned.splitn(3, ':');
  let Some(version) = parts.next() else {
    return Err(SnapshotError::Malformed);
  };
  if version != SNAPSHOT_VERSION {
    return Err(SnapshotError::UnsupportedVersion(version.to_string()));
  }
  let Some(content) = parts.next() else {
    return Err(SnapshotError::Malformed);
  };
  if content != SNAPSHOT_CONTENT {
    return Err(SnapshotError::UnsupportedContent(content.to_string()));
  }
  let Some(payload) = parts.next() else {
    return Err(SnapshotError::Malformed);
  };
  if payload.is_empty() {
    return Ok(Vec::new());
  }
  let commands = payload.split(';');
  if commands.clone().count() > SNAPSHOT_MAX_COMMANDS {
    return Err(SnapshotError::TooLarge);
  }
  commands.map(decode_command).collect()
}

/// Builds one bounded diagnostic record for a rejected browser save.
///
/// The record is never accepted by [`decode_snapshot`]. Keeping the original
/// value when it fits gives a future explicit migration a chance to inspect
/// it, while oversized values are represented by their size rather than
/// allowing localStorage recovery data to grow without bound.
#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn encode_quarantine_record(token: &str, error: &SnapshotError) -> String {
  let error_text: String = error.to_string().chars().take(256).collect();
  let header = format!(
    "{QUARANTINE_PREFIX}bytes={};error={error_text}\n",
    token.len()
  );
  if header.len().saturating_add(token.len()) <= SNAPSHOT_MAX_BYTES {
    return format!("{header}{token}");
  }
  format!("{header}<token omitted: exceeds {SNAPSHOT_MAX_BYTES} bytes>")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn quarantine_record_preserves_small_rejected_tokens() {
    let record = encode_quarantine_record("not-a-snapshot", &SnapshotError::Malformed);
    assert!(record.starts_with("DRL-RUST-BROWSER-REJECTED/1:bytes=14;error="));
    assert!(record.ends_with("\nnot-a-snapshot"));
    assert!(record.len() <= SNAPSHOT_MAX_BYTES);
  }

  #[test]
  fn quarantine_record_bounds_oversized_tokens() {
    let token = "x".repeat(SNAPSHOT_MAX_BYTES * 2);
    let record = encode_quarantine_record(&token, &SnapshotError::TooLarge);
    assert!(record.contains("token omitted"));
    assert!(record.len() <= SNAPSHOT_MAX_BYTES);
  }
}
