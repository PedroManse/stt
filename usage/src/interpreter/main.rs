use stck::prelude::*;

fn main() -> Result<(), stck::Error> {
    let file_path = std::env::args().nth(1).unwrap();
    let mut file_cacher = CacheHelper::new();
    let mut exec_ctx = RuntimeContext::new();
    let code = get_project_code(file_path, &mut file_cacher)?;
    exec_ctx.execute_entire_code(&code)?;
    Ok(())
}
