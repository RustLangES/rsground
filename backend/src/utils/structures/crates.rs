use toml::{map::Map, Value};


pub fn is_toml_allowed(toml_str: impl ToString) -> bool {
    let Ok(parsed_toml) = toml_str.to_string().parse::<Value>()
    else { return false; };

    let allowed = vec![
        "dependencies",
        "dev-dependencies",
        "build-dependencies"
    ];

    fn check_table(table: &Map<String, Value>, allowed: &Vec<&str>) -> bool {
        table.keys().all(|key| {
            if key == "target" {
                if let Some(sub_table) = table.get(key).and_then(Value::as_table) {
                    return sub_table.keys().all(move |target_key| {
                        sub_table.get(target_key)
                            .and_then(Value::as_table)
                            .map(move |t| check_table(t, allowed))
                            .unwrap_or(false)
                    })
                } else {
                    return false;
                }
            }

            allowed.contains(&key.as_str())
        })
    }

    parsed_toml
        .as_table()
        .map(move |t| check_table(t, &allowed))
        .unwrap_or(false)
}
