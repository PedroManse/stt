//! # This module exposes the steps of the pipeline for file execution
//! The steps are:
//! 1. Parsing and preprocessing the file into tokens, avaliable with [get_tokens]
//! 2. Pre-processing the file, also avaliable with [get_tokens]
//! 3. Parsing the tokens into code, avaliable with [get_project_code]
//! 4. Executing the code, avaliable with [execute_file]
//!
//! Each step executes the previous one aswell to forbid jump pipeline steps
//!

use crate::*;

/// # Get tokens by filepath
/// ```rust
/// let tokens = stt::api::get_tokens("examples/test.stt");
/// eprintln!("{:?}", tokens);
/// ```
pub fn get_tokens(path: impl AsRef<Path>) -> Result<TokenBlock> {
    let file_path = PathBuf::from(path.as_ref());
    let tokens = get_raw_tokens(&file_path)?;
    preproc_tokens(tokens, &file_path)
}

/// # Parse tokens into code
/// ```rust
/// let code = stt::api::get_project_code("examples/test.stt");
/// eprintln!("{:?}", code);
/// ```
pub fn get_project_code(path: impl AsRef<Path>) -> Result<Code> {
    let TokenBlock { tokens, source } = get_tokens(path)?;
    let mut parser = parse::Context::new(tokens);
    let exprs = parser.parse_block()?;
    Ok(Code { exprs, source })
}

/// # Execute code
/// ```rust
/// stt::api::execute_file("examples/test.stt");
/// ```
pub fn execute_file(path: impl AsRef<Path>) -> Result<()> {
    let expr_block = get_project_code(path)?;
    execute_code(expr_block)
}

// steps for token.rs:
fn get_raw_tokens(file_path: &Path) -> Result<TokenBlock> {
    let Ok(cont) = std::fs::read_to_string(file_path) else {
        return Err(SttError::CantReadFile(file_path.to_path_buf()));
    };
    let tokens = token::Context::new(&cont).tokenize_block()?;
    Ok(TokenBlock {
        tokens,
        source: file_path.into(),
    })
}

pub(crate) fn get_tokens_with_procvars(
    path: impl AsRef<Path>,
    proc_vars: &mut HashSet<String>,
) -> Result<TokenBlock> {
    let file_path = PathBuf::from(path.as_ref());
    let tokens = get_raw_tokens(&file_path)?;
    preproc_tokens_with_vars(tokens, &file_path, proc_vars)
}

// steps for preproc.rs:
fn preproc_tokens(
    TokenBlock { tokens, source }: TokenBlock,
    file_path: &Path,
) -> Result<TokenBlock> {
    let cwd = PathBuf::from(".");
    let preprocessor = preproc::Context::new(file_path.parent().unwrap_or(cwd.as_path()));
    let tokens = preprocessor.parse_clean(tokens)?;
    Ok(TokenBlock { tokens, source })
}

fn preproc_tokens_with_vars(
    TokenBlock { tokens, source }: TokenBlock,
    file_path: &Path,
    vars: &mut HashSet<String>,
) -> Result<TokenBlock> {
    let cwd = PathBuf::from(".");
    let preprocessor = preproc::Context::new(file_path.parent().unwrap_or(cwd.as_path()));
    let tokens = preprocessor.parse(tokens, vars)?;
    Ok(TokenBlock { tokens, source })
}

// step for runtime:
fn execute_code(code: Code) -> Result<()> {
    let mut executioner = runtime::Context::new();
    executioner.execute_code(&code.exprs, &code.source)?;
    Ok(())
}
