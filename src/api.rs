use crate::*;

// step token.rs
pub fn get_raw_tokens(file_path: &Path) -> Result<TokenBlock> {
    let Ok(cont) = std::fs::read_to_string(file_path) else {
        return Err(SttError::CantReadFile(file_path.to_path_buf()));
    };
    let tokens = token::Context::new(&cont).tokenize_block()?;
    Ok(TokenBlock {
        tokens,
        source: file_path.into(),
    })
}

// step preproc.rs
pub fn preproc_tokens(
    TokenBlock { tokens, source }: TokenBlock,
    file_path: &Path,
) -> Result<TokenBlock> {
    let cwd = PathBuf::from(".");
    let preprocessor = preproc::Context::new(file_path.parent().unwrap_or(cwd.as_path()));
    let tokens = preprocessor.parse_clean(tokens)?;
    Ok(TokenBlock { tokens, source })
}

pub fn preproc_tokens_with_vars(
    TokenBlock { tokens, source }: TokenBlock,
    file_path: &Path,
    vars: &mut HashSet<String>,
) -> Result<TokenBlock> {
    let cwd = PathBuf::from(".");
    let preprocessor = preproc::Context::new(file_path.parent().unwrap_or(cwd.as_path()));
    let tokens = preprocessor.parse(tokens, vars)?;
    Ok(TokenBlock { tokens, source })
}

// step parse.rs
pub fn parse_tokens(TokenBlock { tokens, source }: TokenBlock) -> Result<Code> {
    let mut parser = parse::Context::new(tokens);
    let exprs = parser.parse_block()?;
    Ok(Code { exprs, source })
}

pub fn execute_code(code: Code) -> Result<()> {
    let mut executioner = runtime::Context::new();
    executioner.execute_code(&code.exprs, &code.source)?;
    Ok(())
}

// abstract
pub fn get_tokens(path: impl AsRef<Path>) -> Result<TokenBlock> {
    let file_path = PathBuf::from(path.as_ref());
    let tokens = get_raw_tokens(&file_path)?;
    preproc_tokens(tokens, &file_path)
}

pub fn get_tokens_with_procvars(
    path: impl AsRef<Path>,
    proc_vars: &mut HashSet<String>,
) -> Result<TokenBlock> {
    let file_path = PathBuf::from(path.as_ref());
    let tokens = get_raw_tokens(&file_path)?;
    preproc_tokens_with_vars(tokens, &file_path, proc_vars)
}

pub fn get_project_code(path: impl AsRef<Path>) -> Result<Code> {
    let TokenBlock { tokens, source } = get_tokens(path)?;
    let mut parser = parse::Context::new(tokens);
    let exprs = parser.parse_block()?;
    Ok(Code { exprs, source })
}

pub fn execute_file(path: impl AsRef<Path>) -> Result<()> {
    let expr_block = get_project_code(path)?;
    execute_code(expr_block)
}
