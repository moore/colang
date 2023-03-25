use super::*;

#[test]
fn call_fn () -> Result<(), Error<Rule>> {
    let file = "src/lang/example.co";
    let _result = parse_colang_file(file)?;
    Ok(())
}

/* 
#[test]
fn simple () -> Result<(), Error<Rule>> {
    let file = "src/lang/simple.co";
 
    let _result = parse_colang_file(file)?;
    Ok(())
}
*/