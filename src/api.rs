//! # This module exposes the steps of the pipeline for file execution
//! The steps are:
//! 1. Parsing and preprocessing the file into tokens, avaliable with [`get_tokens`]
//! 2. Pre-processing the file, also avaliable with [`get_tokens`]
//! 3. Parsing the tokens into code, avaliable with [`get_project_code`]
//! 4. Executing the code, avaliable with [`execute_file`]
//!
//! Each step executes the previous one aswell to forbid jump pipeline steps
//!

use crate::*;
type SResult<T> = std::result::Result<T, crate::error::Error>;

/// # Parse tokens from file
/// ```rust
/// let tokens = stck::api::get_tokens("examples/test.stck");
/// eprintln!("{:?}", tokens);
/// ```
pub fn get_tokens(path: impl AsRef<Path>) -> SResult<TokenBlock> {
    let file_path = PathBuf::from(path.as_ref());
    let tokens = get_raw_tokens(&file_path)?;
    preproc_tokens(tokens, &file_path)
}

/// # Parse tokens from string and executes the preprocessor
/// ```rust
/// // "source" of the string mut be annotated
/// // Tokenizer still needs a `\n` in the end of every string (issue #43)
/// let token_block = stck::api::get_tokens_str("\"hello\\n\" print\n", "From raw string").unwrap();
/// assert_eq!(token_block.token_count(), 2);
/// ```
pub fn get_tokens_str(cont: &str, content_name: impl AsRef<Path>) -> SResult<TokenBlock> {
    let tokens = token::Context::new(cont).tokenize(content_name.as_ref().to_path_buf())?;
    preproc_tokens(tokens, content_name.as_ref())
}

/// # Parse code from file
/// ```rust
/// let code = stck::api::get_project_code("examples/test.stck");
/// eprintln!("{:?}", code);
/// ```
pub fn get_project_code(path: impl AsRef<Path>) -> SResult<Code> {
    let TokenBlock {
        tokens,
        source,
        line_breaks,
    } = get_tokens(path)?;
    let mut parser = parse::Context::new(tokens, &source);
    let exprs = parser.parse_block()?;
    Ok(Code {
        line_breaks,
        source,
        exprs,
    })
}

/// # Parse expressions from tokens
/// ```rust
/// let token_block = stck::api::get_tokens_str("\"hello\\n\" print\n", "From raw string").unwrap();
/// # assert_eq!(token_block.token_count(), 2);
/// let code = stck::api::parse_raw_tokens(token_block).unwrap();
/// assert_eq!(code.expr_count(), 2);
/// ```
pub fn parse_raw_tokens(
    TokenBlock {
        line_breaks,
        tokens,
        source,
    }: TokenBlock,
) -> SResult<Code> {
    let mut parser = parse::Context::new(tokens, &source);
    let exprs = parser.parse_block()?;
    Ok(Code {
        line_breaks,
        source,
        exprs,
    })
}

/// # Execute code from file
/// ```rust
/// stck::api::execute_file("examples/test.stck");
/// ```
pub fn execute_file(path: impl AsRef<Path>) -> SResult<()> {
    let expr_block = get_project_code(path)?;
    execute_code(&expr_block)?;
    Ok(())
}

/// # Execute code from expressions
/// ```rust
/// let token_block = stck::api::get_tokens_str("5 2 -\n", "From raw string").unwrap();
/// # assert_eq!(token_block.token_count(), 3);
/// let code = stck::api::parse_raw_tokens(token_block).unwrap();
/// # assert_eq!(code.expr_count(), 3);
/// let ctx = stck::api::execute_raw_code(&code).unwrap();
/// assert_eq!(ctx.get_stack()[0], stck::Value::Num(3));
/// ```
pub fn execute_raw_code(code: &Code) -> Result<runtime::Context, RuntimeError> {
    execute_code(code)
}

fn read_file(file_path: impl AsRef<Path>) -> SResult<String> {
    match std::fs::read_to_string(file_path.as_ref()) {
        Ok(cont) => Ok(cont),
        Err(_) => Err(StckError::CantReadFile(file_path.as_ref().to_path_buf()).into()),
    }
}

// steps for token.rs:
fn get_raw_tokens(file_path: &Path) -> SResult<TokenBlock> {
    let cont = read_file(file_path)?;
    token::Context::new(&cont)
        .tokenize(file_path.to_path_buf())
        .map_err(error::Error::from)
}

pub fn get_tokens_with_procvars<S: std::hash::BuildHasher>(
    path: impl AsRef<Path>,
    proc_vars: &mut HashSet<String, S>,
) -> SResult<TokenBlock> {
    let file_path = PathBuf::from(path.as_ref());
    let tokens = get_raw_tokens(&file_path)?;
    preproc_tokens_with_vars(tokens, &file_path, proc_vars)
}

// steps for preproc.rs:
fn preproc_tokens(
    TokenBlock {
        tokens,
        source,
        line_breaks,
    }: TokenBlock,
    file_path: &Path,
) -> SResult<TokenBlock> {
    let cwd = PathBuf::from(".");
    let preprocessor = preproc::Context::new(file_path.parent().unwrap_or(cwd.as_path()));
    let tokens = preprocessor.parse_clean(tokens)?;
    Ok(TokenBlock {
        line_breaks,
        source,
        tokens,
    })
}

fn preproc_tokens_with_vars<S: std::hash::BuildHasher>(
    TokenBlock {
        tokens,
        source,
        line_breaks,
    }: TokenBlock,
    file_path: &Path,
    vars: &mut HashSet<String, S>,
) -> SResult<TokenBlock> {
    let cwd = PathBuf::from(".");
    let preprocessor = preproc::Context::new(file_path.parent().unwrap_or(cwd.as_path()));
    let tokens = preprocessor.parse(tokens, vars)?;
    Ok(TokenBlock {
        line_breaks,
        source,
        tokens,
    })
}

// step for runtime:
fn execute_code(code: &Code) -> Result<runtime::Context, RuntimeError> {
    let mut executioner = runtime::Context::new();
    executioner.execute_entire_code(code)?;
    Ok(executioner)
}
