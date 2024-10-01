use std::env;

pub(crate) fn tokenize(input: &String) -> Vec<String> {
    let tokens: Vec<&str> = input.trim().split_whitespace().collect();
    let mut result: Vec<String> = Vec::new();
    for token in tokens {
        if token.starts_with('$') {
            result.push(get_env_variable(token[1..].to_string()));
        } else if token.starts_with("~/") || (token == "~" && token.len() == 1) {
            result.push(get_env_variable("HOME".to_string()) + &token[1..])
        } else {
            result.push(token.to_string());
        }
    }
    result
}

pub(crate) fn get_env_variable(input: String) -> String {
    if let Ok(output) = env::var(input) {
        return output;
    }
    "unknown".to_string()
}