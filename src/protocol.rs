use std::io::{self, Write};

pub const GAME_PID_PREFIX: &str = "TANDEM_INTERNAL_GAME_PID=";

pub fn report_game_pid(pid: u32) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{GAME_PID_PREFIX}{pid}")?;
    stdout.flush()
}

pub fn parse_game_pid(line: &str) -> Option<u32> {
    line.strip_prefix(GAME_PID_PREFIX)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::parse_game_pid;

    #[test]
    fn parses_guardian_game_pid_message() {
        assert_eq!(parse_game_pid("TANDEM_INTERNAL_GAME_PID=4242"), Some(4242));
    }

    #[test]
    fn ignores_normal_output() {
        assert_eq!(parse_game_pid("Game process started with PID 4242"), None);
    }

    #[test]
    fn rejects_invalid_pid_message() {
        assert_eq!(parse_game_pid("TANDEM_INTERNAL_GAME_PID=invalid"), None);
    }
}
