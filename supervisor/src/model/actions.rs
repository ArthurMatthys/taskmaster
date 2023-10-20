use std::fmt::Display;

#[derive(Debug)]
pub enum ParseActionError {
    NoCommandFound,
    NoProgramsProvided(String),
    ToManyArguments(String),
    UnrecognizedAction(String),
}

impl Display for ParseActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseActionError::NoCommandFound => writeln!(f, "\x1B[31mNo command found\x1B[0m"),
            ParseActionError::NoProgramsProvided(a) => {
                writeln!(f, "\x1B[31mThe command {:?} need programs to run\x1B[0m", a)
            }
            ParseActionError::ToManyArguments(a) => {
                writeln!(
                    f,
                    "\x1B[31mThe command {:?} doesn't need programs to run\x1B[0",
                    a
                )
            }
            ParseActionError::UnrecognizedAction(a) => {
                writeln!(f, "\x1B[31mThe command {:?} is not recognized\x1B[0m", a)
            }
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Action {
    Quit,
    Reload,
    Restart(Vec<String>),
    Status,
    Start(Vec<String>),
    Stop(Vec<String>),
}

impl ToString for Action {
    fn to_string(&self) -> String {
        match &self {
            Action::Quit => String::from("quit"),
            Action::Reload => String::from("reload"),
            Action::Restart(programs) => format!("restart {}", programs.join(" ")),
            Action::Status => String::from("status"),
            Action::Start(programs) => format!("start {}", programs.join(" ")),
            Action::Stop(programs) => format!("stop {}", programs.join(" ")),
        }
    }
}

impl TryFrom<String> for Action {
    type Error = ParseActionError;

    fn try_from(cmd: String) -> Result<Self, Self::Error> {
        let mut args = cmd.split_whitespace();
        let Some(action) = args.next() else {
            return Err(ParseActionError::NoCommandFound);
        };
        let programs = args.map(|e| e.to_string()).collect::<Vec<String>>();
        let lower_action = action.to_lowercase();
        match lower_action.as_str() {
            "quit" => {
                if programs.is_empty() {
                    Ok(Action::Quit)
                } else {
                    Err(ParseActionError::ToManyArguments(lower_action))
                }
            }
            "reload" => {
                if programs.is_empty() {
                    Ok(Action::Reload)
                } else {
                    Err(ParseActionError::ToManyArguments(lower_action))
                }
            }
            "restart" => {
                if programs.is_empty() {
                    Err(ParseActionError::NoProgramsProvided(lower_action))
                } else {
                    Ok(Action::Restart(programs))
                }
            }
            "status" => {
                if programs.is_empty() {
                    Ok(Action::Status)
                } else {
                    Err(ParseActionError::ToManyArguments(lower_action))
                }
            }
            "start" => {
                if programs.is_empty() {
                    Err(ParseActionError::NoProgramsProvided(lower_action))
                } else {
                    Ok(Action::Start(programs))
                }
            }
            "stop" => {
                if programs.is_empty() {
                    Err(ParseActionError::NoProgramsProvided(lower_action))
                } else {
                    Ok(Action::Stop(programs))
                }
            }
            v => Err(ParseActionError::UnrecognizedAction(v.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quit() -> std::result::Result<(), ParseActionError> {
        let action: Action = String::from("quit").try_into()?;
        assert_eq!(action, Action::Quit);
        let action: Action = String::from("qUit").try_into()?;
        assert_eq!(action, Action::Quit);
        let action: Action = String::from("QUit").try_into()?;
        assert_eq!(action, Action::Quit);
        let action: Action = String::from("qUit ").try_into()?;
        assert_eq!(action, Action::Quit);
        Ok(())
    }
    #[test]
    fn quit_with_args() -> std::result::Result<(), ParseActionError> {
        let action = String::from("quit bonjour");
        match TryInto::<Action>::try_into(action) {
            Err(ParseActionError::ToManyArguments(_)) => assert!(true),
            _ => assert!(false),
        }
        Ok(())
    }
    #[test]
    fn convert_string_1() -> std::result::Result<(), ParseActionError> {
        let action = Action::Reload;
        assert_eq!(action.to_string(), String::from("reload"));
        Ok(())
    }
    #[test]
    fn convert_string_2() -> std::result::Result<(), ParseActionError> {
        let action = Action::Start(vec![String::from("bonjour"), String::from("Hello")]);
        let cpy: Action = action.clone().to_string().try_into()?;
        assert_eq!(cpy, action);
        Ok(())
    }

    #[test]
    fn reload() -> std::result::Result<(), ParseActionError> {
        let action: Action = String::from("reload").try_into()?;
        assert_eq!(action, Action::Reload);
        let action: Action = String::from("Reload").try_into()?;
        assert_eq!(action, Action::Reload);
        let action: Action = String::from("RELOAD").try_into()?;
        assert_eq!(action, Action::Reload);
        let action: Action = String::from("reLoad").try_into()?;
        assert_eq!(action, Action::Reload);
        Ok(())
    }
    #[test]
    fn reload_with_args() -> std::result::Result<(), ParseActionError> {
        let action = String::from("reload bonjour");
        match TryInto::<Action>::try_into(action) {
            Err(ParseActionError::ToManyArguments(_)) => assert!(true),
            _ => assert!(false),
        }
        Ok(())
    }
    #[test]
    fn restart() -> std::result::Result<(), ParseActionError> {
        let action: Action = String::from("restart blabla").try_into()?;
        assert_eq!(action, Action::Restart(vec!["blabla".to_string()]));
        let action: Action = String::from("Restart blabla").try_into()?;
        assert_eq!(action, Action::Restart(vec!["blabla".to_string()]));
        let action: Action = String::from("restArt blabla").try_into()?;
        assert_eq!(action, Action::Restart(vec!["blabla".to_string()]));
        let action: Action = String::from("RESTART blabla").try_into()?;
        assert_eq!(action, Action::Restart(vec!["blabla".to_string()]));
        Ok(())
    }
    #[test]
    fn restart_without_args() -> std::result::Result<(), ParseActionError> {
        let action = String::from("restart");
        match TryInto::<Action>::try_into(action) {
            Err(ParseActionError::NoProgramsProvided(_)) => assert!(true),
            _ => assert!(false),
        }
        Ok(())
    }
    #[test]
    fn unknown_command() -> std::result::Result<(), ParseActionError> {
        let cmd = String::from("Bonjour");
        match TryInto::<Action>::try_into(cmd) {
            Err(ParseActionError::UnrecognizedAction(_)) => assert!(true),
            _ => assert!(false),
        }
        Ok(())
    }
}
