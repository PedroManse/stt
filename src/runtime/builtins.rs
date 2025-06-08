use super::*;

pub(super) fn sh(shell_cmd: &str) -> OResult<isize, String> {
    eprintln!("[CMD] {shell_cmd}");
    Ok(0)
    //std::proces::Command::new("bash")
    //    .arg("-c")
    //    .arg(shell_cmd)
    //    .status()
    //    .map(|s| s.code().unwrap_or(256) as isize)
    //    .map_err(|e| e.to_string())
}
pub(super) fn write_to(cont: &str, file: &str) -> OResult<isize, String> {
    eprintln!("Write {} bytes to {file}", cont.len());
    Ok(cont.len() as isize)
}

enum FmtError {
    MissingValue(char),
    UnknownStringFormat(char),
    WrongVariableForFormat(Value, char),
}

fn fmt_internal(cont: &str, stack: &mut Stack) -> OResult<String, FmtError> {
    let mut out = String::with_capacity(cont.len());
    enum State {
        Nothing,
        OnFmt,
    }
    let mut state = State::Nothing;
    for ch in cont.chars() {
        state = match (state, ch) {
            (State::Nothing, '%') => State::OnFmt,
            (State::Nothing, ch) => {
                out.push(ch);
                State::Nothing
            }
            (State::OnFmt, '%') => {
                out.push('%');
                State::Nothing
            }
            (State::OnFmt, 's') => {
                let add_str = stack_pop!(=(stack) -> str? as "%s" for "%%")
                    .ok_or(FmtError::MissingValue('s'))?
                    .map_err(|v| FmtError::WrongVariableForFormat(v, 's'))?;
                out.push_str(&add_str);
                State::Nothing
            }
            (State::OnFmt, 'd') => {
                let add_num = stack_pop!(=(stack) -> num? as "%d" for "%%")
                    .ok_or(FmtError::MissingValue('d'))?
                    .map_err(|v| FmtError::WrongVariableForFormat(v, 'd'))?;
                out.push_str(&add_num.to_string());
                State::Nothing
            }
            (State::OnFmt, 'v') => {
                let fmt = match stack.pop() {
                    Some(x) => format!("{x:?}"),
                    None => "<Nothing in stack>".to_string(),
                };
                out.push_str(&fmt);
                State::Nothing
            }
            (State::OnFmt, 'b') => {
                let add_bool = stack_pop!(=(stack) -> bool? as "%b" for "%%")
                    .ok_or(FmtError::MissingValue('b'))?
                    .map_err(|v| FmtError::WrongVariableForFormat(v, 'b'))?;
                out.push_str(&add_bool.to_string());
                State::Nothing
            }
            (State::OnFmt, x) => {
                return Err(FmtError::UnknownStringFormat(x));
            }
        }
    }
    Ok(out)
}

pub(super) fn fmt(cont: &str, stack: &mut Stack) -> Result<String> {
    fmt_internal(cont, stack).map_err(|e| {
        let fmt_str = cont.to_string();
        match e {
            FmtError::MissingValue(c) => SttError::RTMissingValue(fmt_str, c),
            FmtError::UnknownStringFormat(c) => SttError::RTUnknownStringFormat(fmt_str, c),
            FmtError::WrongVariableForFormat(v, c) => SttError::RTWrongValueType(fmt_str, v, c),
        }
    })
}
